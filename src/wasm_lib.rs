// クレーと外部からでもモジュールを使用できるようにするためのファイル。
// 外部からこのクレートライブラリをImportした時、使用できる機能を公開している。

#![cfg_attr(feature = "custom-alloc", feature(allocator_api))]

#[cfg(feature = "custom-alloc")]
mod alloc;

#[cfg(feature = "bincode")]
mod file;

mod action_tree;
mod atomic_float;
mod bet_size;
mod bunching;
mod card;
mod game;
mod hand;
mod hand_table;
mod interface;
mod mutex_like;
mod range;
mod sliceop;
mod solver;
mod utility;

use anyhow::Error;
#[cfg(feature = "bincode")]
pub use file::*;
use std::str::FromStr;

use crate::{
    range::card_from_str,
    utility::{compute_average, finalize},
};
use action_tree::{Action, ActionTree, TreeConfig};
use bet_size::BetSizeOptions;
use card::{Card, CardConfig};
use game::PostFlopGame;
use range::{flop_from_str, hole_to_string, Range};

#[allow(warnings)]
mod bindings;
use bindings::exports::holdem_solver::host::game_manager;
// use bindings::exports::holdem_solver::host::my_host;
// use chrono::Local;
use std::cell::Cell;

pub struct MyGame {
    game: std::cell::RefCell<PostFlopGame>,
}

impl From<Action> for game_manager::WitAction {
    fn from(action: Action) -> Self {
        match action {
            Action::None => game_manager::WitAction::None,
            Action::Fold => game_manager::WitAction::Fold,
            Action::Check => game_manager::WitAction::Check,
            Action::Call => game_manager::WitAction::Call,
            Action::Bet(x) => game_manager::WitAction::Bet(x as u32),
            Action::Raise(x) => game_manager::WitAction::Raise(x as u32),
            Action::AllIn(x) => game_manager::WitAction::AllIn(x as u32),
            Action::Chance(card) => game_manager::WitAction::Chance(card as u32),
        }
    }
}

impl game_manager::GuestGameResource for MyGame {
    fn new() -> game_manager::GameResource {
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
            flop: flop_from_str("Td9d6h").unwrap(),
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

    fn check_action(&self) -> Result<bool, String> {
        let mut mut_game = self.game.borrow_mut();
        if mut_game.is_chance_node() {
            Err("This action is invalid.".to_string())
        } else {
            mut_game.play(0); // Check
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

    fn get_history(&self) -> Vec<u32> {
        let mut_game: std::cell::RefMut<'_, PostFlopGame> = self.game.borrow_mut();
        let history_usize: &[usize] = mut_game.history();
        history_usize.into_iter().map(|c| *c as u32).collect()
    }

    fn get_strategy(&self) -> Vec<f32> {
        let mut_game: std::cell::RefMut<'_, PostFlopGame> = self.game.borrow_mut();
        let strategy = mut_game.strategy();
        strategy
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

// bindings::export!(MyFunction with_types_in bindings);
bindings::export!(MyGame with_types_in bindings);
