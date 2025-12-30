// クレーと外部からでもモジュールを使用できるようにするためのファイル。
// 外部からこのクレートライブラリをImportした時、使用できる機能を公開している。

#![cfg_attr(feature = "custom-alloc", feature(allocator_api))]

mod action_tree;
mod atomic_float;
mod bet_size;
mod bunching;
mod card;
mod file;
mod game;
mod hand;
mod hand_table;
mod interface;
mod mutex_like;
mod range;
mod sliceop;
mod solver;
mod utility;
mod wit_models;

use core::panic;
use solver::solve_step;
use std::{collections::HashMap, str::FromStr};

use crate::{
    game::ActionHistoryDetail,
    range::{card_from_str, card_to_string},
    utility::{compute_average, compute_exploitability, finalize},
};
use action_tree::{Action, ActionTree, TreeConfig};
use bet_size::BetSizeOptions;
use card::CardConfig;
use file::{save_data_into_std_write, save_data_to_file};
use game::PostFlopGame;
use range::{flop_from_str, hole_to_string, Range};

#[allow(warnings)]
mod bindings;
use crate::bindings::exports::holdem_solver::host::game_manager;
// use bindings::exports::holdem_solver::host::my_host;
// use chrono::Local;

pub struct MyGame {
    game: std::cell::RefCell<PostFlopGame>,
}

impl game_manager::GuestGameResource for MyGame {
    fn new(flop_card_str: String, mode: u8) -> game_manager::GameResource {
        println!("Initial Proccess Start Running!!");
        let betsite_option;
        if mode == 0 {
            betsite_option = BetSizeOptions::try_from(("33%,70%", "")).unwrap();
        } else if mode == 1 {
            betsite_option = BetSizeOptions::try_from(("33%,75%,a", "3x,a")).unwrap();
        } else if mode == 2 {
            betsite_option =
                BetSizeOptions::try_from(("15%,33%,50%,75%,100%,150%,a", "2.5x,3x,3.5x,4x,a"))
                    .unwrap();
        } else {
            panic!("Invalid mode");
        }
        let betsize_option = betsite_option;
        // GameResource::new(...) はバインディング生成に含まれるスマートポインタ型
        let card_config: CardConfig = CardConfig {
            range: [
                Range::from_str("AA-JJ,AQo+,KQo+,AKs-AQs,A5s").unwrap(), // OOP(UTG 4bet)
                Range::from_str("QQ-88,AKo,AQs+,KJs+,QJs+,65s").unwrap(), // IP (BTN 4bet caller)
            ],
            flop: flop_from_str(flop_card_str.as_str()).unwrap(),
            ..Default::default()
        };

        let tree_config = TreeConfig {
            starting_pot: 16,
            effective_stack: 100,
            flop_bet_sizes: [betsize_option.clone(), betsize_option.clone()],
            turn_bet_sizes: [betsize_option.clone(), betsize_option.clone()],
            river_bet_sizes: [betsize_option.clone(), betsize_option.clone()],
            ..Default::default()
        };

        let action_tree = ActionTree::new(tree_config).unwrap();
        let mut game: PostFlopGame = PostFlopGame::with_config(card_config, action_tree).unwrap();
        game.allocate_memory(true);

        //// ↓ solve_stepロジックの内部実装。今後、外部関数化、もしくはWasm関数にすること。
        let max_iterations = 100;
        let target_exploitability = 10.0;

        for iteration in 0..max_iterations {
            // 1イテレーション実行
            solve_step(&game, iteration);

            // 3回ごとにexploitabilityを計算して進捗を確認
            if (iteration + 1) % 3 == 0 {
                let exploitability = compute_exploitability(&game);
                println!(
                    "Iteration: {}, Exploitability: {:.6e}",
                    iteration + 1,
                    exploitability
                );

                if exploitability <= target_exploitability {
                    break;
                }
            }
        }
        //// ↑ solve関数ここまで

        finalize(&mut game); // 演算をしているっぽい。
        game.cache_normalized_weights();
        // save_data_to_file(&game, "メモ", "game.flop", Some(3)); // 動かないが理由もよくわからない
        game_manager::GameResource::new(Self {
            game: std::cell::RefCell::new(game),
        })
    }

    fn from_cache(cache: Vec<u8>) -> Result<game_manager::GameResource, String> {
        let game: PostFlopGame =
            file::load_data_from_std_read(&mut &*cache, Some(isize::MAX as u64))?.0;
        Ok(game_manager::GameResource::new(Self {
            game: std::cell::RefCell::new(game),
        }))
    }

    fn card_deal(&self, card_str: String) -> Result<bool, String> {
        let mut mut_game = self.game.borrow_mut();
        let card_char: &str = &card_str;
        let card = card_from_str(card_char).unwrap() as usize;
        if mut_game.is_chance_node() {
            mut_game.play(card); // Deal
            mut_game.cache_normalized_weights();
            Ok(true)
        } else {
            Err("This action is invalid.".to_string()) // ERROR
        }
    }

    fn action(&self, action_num: u32) -> Result<bool, String> {
        let mut mut_game = self.game.borrow_mut();
        mut_game.play(action_num as usize); // Check
        mut_game.cache_normalized_weights();
        Ok(true)
    }

    fn get_range(&self, player: u32) -> Vec<f32> {
        let mut_game = self.game.borrow_mut();
        let range = mut_game.card_config().range[player as usize];
        range.raw_data().to_vec()
    }

    fn get_card_wights(&self, player: u32) -> Vec<(String, f32)> {
        let mut cards_weights: Vec<(String, f32)> = Vec::new();
        let mut_game = self.game.borrow_mut();
        let range = mut_game.card_config().range[player as usize];
        let weights = range.get_hands_weights(0);
        for (hands, weight) in weights.0.iter().zip(weights.1.iter()) {
            let hands_str = hole_to_string(*hands).unwrap();
            cards_weights.push((hands_str, *weight));
        }
        cards_weights
    }

    fn get_node_info(&self) -> Vec<f32> {
        let mut_game = self.game.borrow();
        let weights_oop = mut_game.normalized_weights(0);
        let weights_ip = mut_game.normalized_weights(1);
        let equity_oop = compute_average(&mut_game.equity(0), weights_oop);
        let equity_ip = compute_average(&mut_game.equity(1), weights_ip);
        let ev_oop = compute_average(&mut_game.expected_values(0), weights_oop);
        let ev_ip = compute_average(&mut_game.expected_values(1), weights_ip);
        vec![equity_oop, equity_ip, ev_oop, ev_ip]
    }

    fn get_board_cards(&self) -> Vec<String> {
        let mut_game = self.game.borrow_mut();
        let borad_cards = mut_game.current_board();
        let borad_cards_string: Vec<String> = borad_cards
            .into_iter()
            .map(|c| card_to_string(c).unwrap())
            .collect();
        borad_cards_string
    }

    fn get_available_action(&self) -> Vec<game_manager::WitAction> {
        let mut_game = self.game.borrow_mut();
        let actions = mut_game.available_actions();
        let wit_action: Vec<game_manager::WitAction> =
            actions.iter().map(|c| c.clone().into()).collect();
        wit_action
    }

    fn apply_history(&self, history: Vec<u32>) -> Result<bool, ()> {
        let mut mut_game = self.game.borrow_mut();
        let history_usize: Vec<usize> = history.into_iter().map(|c| c as usize).collect();
        let history_usize_address: &[usize] = history_usize.as_slice();
        mut_game.apply_history(history_usize_address);
        mut_game.cache_normalized_weights();
        Ok(true)
    }

    fn get_history(&self) -> Vec<u32> {
        let mut_game: std::cell::RefMut<'_, PostFlopGame> = self.game.borrow_mut();
        let history_usize: &[usize] = mut_game.history();
        history_usize.into_iter().map(|c| *c as u32).collect()
    }

    fn get_valid_actions_history(&self) -> Vec<game_manager::WitActionHistoryDetail> {
        let mut_game: std::cell::RefMut<'_, PostFlopGame> = self.game.borrow_mut();
        let valid_actions_history: Vec<ActionHistoryDetail> = mut_game.get_valid_actions_history();
        let valid_actions_history_wit_actions: Vec<game_manager::WitActionHistoryDetail> =
            valid_actions_history
                .into_iter()
                .map(|detail| game_manager::WitActionHistoryDetail::from(detail))
                .collect();
        valid_actions_history_wit_actions
    }

    fn get_strategy(&self) -> Vec<game_manager::StrategyMap> {
        let mut strategy_list: Vec<game_manager::StrategyMap> = [].to_vec();
        let mut_game: std::cell::RefMut<'_, PostFlopGame> = self.game.borrow_mut();
        let strategy_with_hands: HashMap<String, HashMap<Action, (f32, f32)>> =
            mut_game.strategy_with_hands().unwrap();
        for (hand, strategy) in strategy_with_hands.iter() {
            for (action, (weight, action_ratio)) in strategy.iter() {
                strategy_list.push(game_manager::StrategyMap {
                    hand: hand.clone(),
                    action: game_manager::WitAction::from(*action),
                    strategy: game_manager::Strategy {
                        weight: *weight,
                        action_ratio: *action_ratio,
                    },
                });
            }
        }
        strategy_list
    }

    fn get_game_status(&self) -> game_manager::WitGameStatus {
        let mut_game: std::cell::RefMut<'_, PostFlopGame> = self.game.borrow_mut();
        if mut_game.is_chance_node() {
            return game_manager::WitGameStatus::Chance;
        } else if mut_game.is_terminal_node() {
            return game_manager::WitGameStatus::Terminal;
        } else {
            let current_player = mut_game.current_player();
            if current_player == 0 {
                return game_manager::WitGameStatus::OopAction;
            } else if current_player == 1 {
                return game_manager::WitGameStatus::IpAction;
            } else {
                panic!("Invalid current player");
            }
        }
    }

    fn get_card_index_from_str(&self, card_str: String) -> Result<u32, String> {
        let card_char: &str = &card_str;
        match card_from_str(card_char) {
            Ok(card) => Ok(card as u32),
            Err(e) => Err(format!("Error parsing card string: {}", e)),
        }
    }

    // fn get_game_status(&self) -> String {
    //     mutex_guaid_game = self.game.root();
    //     if mutex_guaid_game.is_terminal() {}
    // }

    fn get_compressed_result(&self) -> Result<Vec<u8>, String> {
        let mut_game = self.game.borrow_mut();
        // 圧縮してバッファに保存
        let mut buffer = Vec::new();
        save_data_into_std_write(&*mut_game, "solved game", &mut buffer, Some(3))?;

        // メモリをJavaScriptに渡す（解放はJavaScript側で行う）
        Ok(buffer)
    }
}

// ② interface 全体 (Guest)
impl game_manager::Guest for MyGame {
    type GameResource = Self;
}

struct WasmUtils {}
impl WasmUtils {
    fn weighted_average(slice: &[f32], weights: &[f32]) -> f64 {
        let mut sum = 0.0;
        let mut weight_sum = 0.0;
        for (&value, &weight) in slice.iter().zip(weights.iter()) {
            sum += value as f64 * weight as f64;
            weight_sum += weight as f64;
        }
        sum / weight_sum
    }
}

// bindings::export!(MyFunction with_types_in bindings);
bindings::export!(MyGame with_types_in bindings);
