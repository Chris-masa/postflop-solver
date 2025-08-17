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
use bindings::exports::holdem_solver::host::game_manager;
// use bindings::exports::holdem_solver::host::my_host;
// use chrono::Local;
use std::cell::Cell;

// pub struct MyFunction;

// impl my_host::Guest for MyFunction {
//     fn get_date() -> String {
//         let now = Local::now();
//         now.format("%Y-%m-%d %H:%M:%S").to_string()
//     }
// }

pub struct MyGame {
    number: Cell<u32>,
}

impl game_manager::GuestGameResource for MyGame {
    // type GameResource = game_manager::GuestGameResource;

    fn new(number: u32) -> game_manager::GameResource {
        // GameResource::new(...) はバインディング生成に含まれるスマートポインタ型
        game_manager::GameResource::new(MyGame {
            number: Cell::new(number),
        })
    }

    fn write(&self, number: u32) {
        self.number.set(number);
    }

    fn read(&self) -> u32 {
        self.number.get()
    }

    fn up(&self) -> u32 {
        self.number.set(self.number.get() + 1);
        self.number.get()
    }

    fn down(&self) -> u32 {
        if self.number.get() == 0 {
            return 0; // Avoid underflow
        } else if self.number.get() == 65535 {
            return 65535; // Avoid underflow
        }
        self.number.set(self.number.get() - 1);
        self.number.get()
    }
}

// ② interface 全体 (Guest)
impl game_manager::Guest for MyGame {
    type GameResource = Self;
}

// bindings::export!(MyFunction with_types_in bindings);
bindings::export!(MyGame with_types_in bindings);
