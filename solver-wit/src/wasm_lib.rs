// クレーと外部からでもモジュールを使用できるようにするためのファイル。
// 外部からこのクレートライブラリをImportした時、使用できる機能を公開している。

mod wit_models;

use solver_core::{PostFlopGameInterface, PostFlopGameTrait};

#[allow(warnings)]
mod bindings;
use crate::bindings::exports::holdem_solver::host::game_manager;
// use bindings::exports::holdem_solver::host::my_host;
// use chrono::Local;

pub struct WitGame {
    wit_game: PostFlopGameInterface,
}

impl game_manager::GuestGameResource for WitGame {
    fn new(flop_card_str: String, mode: u8) -> game_manager::GameResource {
        let wit_game: PostFlopGameInterface = PostFlopGameInterface::new(flop_card_str, mode);
        game_manager::GameResource::new(Self { wit_game: wit_game })
    }

    fn from_cache(cache: Vec<u8>) -> Result<game_manager::GameResource, String> {
        let wit_game: PostFlopGameInterface = PostFlopGameInterface::from_cache(cache)?;
        Ok(game_manager::GameResource::new(Self { wit_game: wit_game }))
    }

    fn card_deal(&self, card_str: String) -> Result<bool, String> {
        PostFlopGameInterface::card_deal(&self.wit_game, card_str)
    }

    fn action(&self, action_num: u32) -> Result<bool, String> {
        PostFlopGameInterface::action(&self.wit_game, action_num)
    }

    fn get_range(&self, player: u32) -> Vec<f32> {
        PostFlopGameInterface::get_range(&self.wit_game, player as usize)
    }

    fn get_card_wights(&self, player: u32) -> Vec<(String, f32)> {
        PostFlopGameInterface::get_card_wights(&self.wit_game, player as usize)
    }

    fn get_node_info(&self) -> Vec<f32> {
        PostFlopGameInterface::get_node_info(&self.wit_game)
    }

    fn get_board_cards(&self) -> Vec<String> {
        PostFlopGameInterface::get_board_cards(&self.wit_game)
    }

    fn get_available_action(&self) -> Vec<game_manager::WitAction> {
        let actions = PostFlopGameInterface::get_available_action(&self.wit_game);
        let wit_action: Vec<game_manager::WitAction> =
            actions.iter().map(|c| c.clone().into()).collect();
        wit_action
    }

    fn apply_history(&self, history: Vec<u32>) -> Result<bool, ()> {
        PostFlopGameInterface::apply_history(&self.wit_game, history)
    }

    fn get_history(&self) -> Vec<u32> {
        PostFlopGameInterface::get_history(&self.wit_game)
    }

    fn get_valid_actions_history(&self) -> Vec<game_manager::WitActionHistoryDetail> {
        let valid_actions_history =
            PostFlopGameInterface::get_valid_actions_history(&self.wit_game);
        let valid_actions_history_wit_actions: Vec<game_manager::WitActionHistoryDetail> =
            valid_actions_history
                .into_iter()
                .map(|detail| game_manager::WitActionHistoryDetail::from(detail))
                .collect();
        valid_actions_history_wit_actions
    }

    fn get_strategy(&self) -> Vec<game_manager::WitStrategyMap> {
        let mut strategy_list: Vec<game_manager::WitStrategyMap> = [].to_vec();
        let strategy_with_hands = PostFlopGameInterface::get_strategy(&self.wit_game);
        for strategy_map in strategy_with_hands.iter() {
            let hand = strategy_map.hand.clone();
            let strategy = strategy_map.strategy.clone();
            let action = strategy_map.action.clone();
            strategy_list.push(game_manager::WitStrategyMap {
                hand: hand.clone(),
                action: game_manager::WitAction::from(action),
                strategy: game_manager::WitStrategy {
                    weight: strategy.weight,
                    action_ratio: strategy.action_ratio,
                },
            });
        }
        strategy_list
    }

    fn get_game_status(&self) -> game_manager::WitGameStatus {
        let game_status = PostFlopGameInterface::get_game_status(&self.wit_game);
        game_manager::WitGameStatus::from(game_status)
    }

    fn get_card_index_from_str(&self, card_str: String) -> Result<u32, String> {
        PostFlopGameInterface::get_card_index_from_str(&self.wit_game, card_str)
    }

    // fn get_game_status(&self) -> String {
    //     mutex_guaid_game = self.wit_game.root();
    //     if mutex_guaid_game.is_terminal() {}
    // }

    fn get_compressed_result(&self) -> Result<Vec<u8>, String> {
        PostFlopGameInterface::get_compressed_result(&self.wit_game)
    }
}

// ② interface 全体 (Guest)
impl game_manager::Guest for WitGame {
    type GameResource = Self;
}

// struct WasmUtils {}
// impl WasmUtils {
//     fn weighted_average(slice: &[f32], weights: &[f32]) -> f64 {
//         let mut sum = 0.0;
//         let mut weight_sum = 0.0;
//         for (&value, &weight) in slice.iter().zip(weights.iter()) {
//             sum += value as f64 * weight as f64;
//             weight_sum += weight as f64;
//         }
//         sum / weight_sum
//     }
// }

// bindings::export!(MyFunction with_types_in bindings);
bindings::export!(WitGame with_types_in bindings);
