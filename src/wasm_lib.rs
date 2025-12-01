// クレーと外部からでもモジュールを使用できるようにするためのファイル。
// 外部からこのクレートライブラリをImportした時、使用できる機能を公開している。

use std::{collections::HashMap, str::FromStr};

use crate::{
    action_tree::{Action, ActionTree, TreeConfig},
    bet_size::BetSizeOptions,
    card::CardConfig,
    game::{ActionHistoryDetail, PostFlopGame},
    range::{card_from_str, flop_from_str, hole_to_string, Range},
    utility::{compute_average, finalize},
    wit_models::wit_conversation::*,
};

use crate::bindings::export;
#[allow(warnings)]
use crate::bindings::exports::holdem_solver::host::game_manager;
// use chrono::Local;

pub struct MyGame {
    game: std::cell::RefCell<PostFlopGame>,
}

impl game_manager::GuestGameResource for MyGame {
    fn new(flop_card_str: String) -> game_manager::GameResource {
        println!("Initial Proccess Start Running!!");
        let betsize_option =
            // BetSizeOptions::try_from(("15%,33%,50%,75%,100%,150%,a", "2.5x,3x,3.5x,4x,a")).unwrap();
            BetSizeOptions::try_from(("33%", "")).unwrap();
        // GameResource::new(...) はバインディング生成に含まれるスマートポインタ型
        let card_config: CardConfig = CardConfig {
            range: [
                Range::from_str("77+,AQo+,KJo+,A4s+,KTs+,QTs+,JTs+").unwrap(),
                Range::from_str("22+,A2s+,K2s+,Q5s+,J7s+,T8s+,98s+").unwrap(),
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
        finalize(&mut game); // 演算をしているっぽい。
        game.cache_normalized_weights();
        game.add_flop_fistory_detail(); // フロップのHistoryは手動追加になってしまっている
        game_manager::GameResource::new(Self {
            game: std::cell::RefCell::new(game),
        })
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
        if mut_game.is_chance_node() {
            Err("This action is invalid.".to_string())
        } else {
            mut_game.play(action_num as usize); // Check
            mut_game.cache_normalized_weights();
            Ok(true)
        }
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
        let borad_cards_string: Vec<String> =
            borad_cards.into_iter().map(|c| c.to_string()).collect();
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
        let history_usize: &[usize] = unsafe {
            std::slice::from_raw_parts(
                history.as_ptr() as *const usize,
                history.len() * std::mem::size_of::<u32>(),
            )
        };
        mut_game.apply_history(history_usize);
        mut_game.cache_normalized_weights();
        Ok(true)
    }

    fn get_card_index_from_str(&self, card_str: String) -> Result<u32, String> {
        let card_char: &str = &card_str;
        match card_from_str(card_char) {
            Ok(card) => Ok(card as u32),
            Err(e) => Err(format!("Error parsing card string: {}", e)),
        }
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

    // fn get_game_status(&self) -> String {
    //     mutex_guaid_game = self.game.root();
    //     if mutex_guaid_game.is_terminal() {}
    // }
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

// bindings::export!(MyGame with_types_in bindings);
export!(MyGame with_types_in crate::bindings);
