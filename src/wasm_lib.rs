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

#[cfg(feature = "bincode")]
pub use file::*;

pub use action_tree::*;
pub use bet_size::*;
pub use bunching::*;
pub use card::*;
pub use game::*;
pub use interface::*;
pub use mutex_like::*;
pub use range::*;
pub use solver::*;
pub use utility::*;

#[allow(warnings)]
mod bindings;

use bindings::exports::holdem_solver::host::my_host::Guest;
use std::cmp::Ordering;

// bindgen!({
//     world:"holdem-solver",// コード生成を行うワールドの名前
//     path: "./wit/holdem-solver", // WITファイルが存在するフォルダーへのパス
// });

use chrono::Local;

pub struct MyFunction;

impl Guest for MyFunction {
    fn get_date() -> String {
        let now = Local::now();
        now.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

// pub struct RangeManager {
//     range: Range,
// }

// impl RangeManager for GuestRangeManager {
//     fn new() -> Self {
//         Self {
//             range: Range::new(),
//         }
//     }

//     fn clear(&mut self) {
//         self.range.clear();
//     }

//     fn update(&mut self, row: u8, col: u8, weight: f32) {
//         let rank1 = 13 - row;
//         let rank2 = 13 - col;
//         match row.cmp(&col) {
//             Ordering::Equal => self.range.set_weight_pair(rank1, weight),
//             Ordering::Less => self.range.set_weight_suited(rank1, rank2, weight),
//             Ordering::Greater => self.range.set_weight_offsuit(rank1, rank2, weight),
//         }
//     }

//     fn from_string(&mut self, s: &str) -> Option<String> {
//         let result = Range::from_sanitized_str(s);
//         if let Ok(unwrap) = result {
//             self.range = unwrap;
//             None
//         } else {
//             result.err()
//         }
//     }

//     fn to_string(&self) -> String {
//         self.range.to_string()
//     }

//     fn get_weights(&self) -> Box<[f32]> {
//         let mut weights = vec![0.0; 13 * 13];

//         for row in 0..13 {
//             for col in 0..13 {
//                 let rank1 = 12 - row as u8;
//                 let rank2 = 12 - col as u8;
//                 weights[row * 13 + col] = match row.cmp(&col) {
//                     Ordering::Equal => self.range.get_weight_pair(rank1),
//                     Ordering::Less => self.range.get_weight_suited(rank1, rank2),
//                     Ordering::Greater => self.range.get_weight_offsuit(rank1, rank2),
//                 };
//             }
//         }

//         weights.into()
//     }

//     fn raw_data(&self) -> Box<[f32]> {
//         self.range.raw_data().into()
//     }
// }

bindings::export!(MyFunction with_types_in bindings);
