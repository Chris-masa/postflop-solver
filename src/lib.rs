// src/lib.rs

// === 純粋ロジック（ネイティブ / WASM 共通） ==================
pub mod action_tree;
pub mod alloc;
pub mod atomic_float;
pub mod bet_size;
pub mod bunching;
pub mod card;
pub mod file;
pub mod hand;
pub mod hand_table;
pub mod interface;
pub mod mutex_like;
pub mod range;
pub mod sliceop;
pub mod solver;
pub mod utility;

// サブモジュール（フォルダ）
pub mod game;

// === WASM / WIT 用エントリ ==============================
// wasm32 ターゲットのときだけコンパイルする
pub mod bindings;
#[cfg(target_arch = "wasm32")]
pub mod wasm_lib;
pub mod wit_models;
