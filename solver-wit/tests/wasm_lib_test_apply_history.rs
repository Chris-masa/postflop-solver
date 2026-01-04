use anyhow::{anyhow, Result};
use exports::holdem_solver::host::game_manager::WitGameStatus::{
    Chance, IpAction, OopAction, Terminal,
};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

// WITから型付きバインディング生成
wasmtime::component::bindgen!({
    path: "wit",
    world: "host",
});

// WASI p2 用の Store state
struct Ctx {
    wasi: WasiCtx,
    table: ResourceTable,
}

impl WasiView for Ctx {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

#[test]
fn wasm_lib_test_apply_history() -> Result<()> {
    let mut cfg = Config::new();
    cfg.wasm_component_model(true);
    let engine = Engine::new(&cfg)?;

    // WASI(p2) をリンク
    let mut linker: Linker<Ctx> = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    // Store 準備
    let wasi = WasiCtx::builder().inherit_stdio().inherit_args().build();
    let mut store = Store::new(
        &engine,
        Ctx {
            wasi,
            table: ResourceTable::new(),
        },
    );

    // Component をロード（ここに余計なコードを混ぜない）
    let component = Component::from_file(
        &engine,
        "../target/wasm32-wasip2/release/postflop_solver.wasm",
    )?;

    // instantiate（※あなたの環境では Host_ が返る）
    let world = Host_::instantiate(&mut store, &component, &linker)?;

    // ✅ ここがポイント：
    // export された interface は `world.<package>_<interface>()` みたいなメソッドとして生えます
    // （メソッド名が微妙に違う場合は、コンパイルエラーに候補が出るのでそれに合わせてください）
    let gm = world.holdem_solver_host_game_manager();

    // resource の投影を取得
    let gr = gm.game_resource();
    println!("GameResource obtained.");

    // WIT: `new: static func(...) -> game-resource;`
    // → wasmtime の生成では `call_new`
    let game = gr.call_new(&mut store, "AsKsQs", 1)?;
    println!("Game created.");

    // get-game-status: func() -> wit-game-status
    let status = gr.call_get_game_status(&mut store, game)?;
    assert_eq!(status, OopAction);

    // カードIndex取得テスト
    let mut card = gr
        .call_get_card_index_from_str(&mut store, game, "2h")
        .unwrap()
        .unwrap();
    println!("Card Index for 2h: {}", card);
    assert_eq!(card, 2); // 2h = 1
    card = gr
        .call_get_card_index_from_str(&mut store, game, "As")
        .unwrap()
        .unwrap();
    println!("Card Index for As: {}", card);
    assert_eq!(card, 51); // As = 51
    card = gr
        .call_get_card_index_from_str(&mut store, game, "Js")
        .unwrap()
        .unwrap();
    println!("Card Index for Js: {}", card);
    assert_eq!(card, 39); // Td = 33

    // OOP Action 1
    let check: bool = gr.call_action(&mut store, game, 0).unwrap().unwrap();
    assert!(check);
    let status = gr.call_get_game_status(&mut store, game)?;
    assert_eq!(
        status,
        exports::holdem_solver::host::game_manager::WitGameStatus::IpAction
    );

    // IP Action 1
    let check: bool = gr.call_action(&mut store, game, 0).unwrap().unwrap();
    assert!(check);
    let status = gr.call_get_game_status(&mut store, game)?;
    assert_eq!(
        status,
        exports::holdem_solver::host::game_manager::WitGameStatus::Chance
    );

    let now_histry_1 = gr.call_get_history(&mut store, game)?;
    assert_eq!(now_histry_1, [0, 0]);

    println!("--- apply history test ---");

    // apply history
    let history: Vec<u32> = vec![1, 1, 30, 0, 1, 0]; // OOP: bet, IP: call
    gr.call_apply_history(&mut store, game, &history)?;
    let now_histry_2 = gr.call_get_history(&mut store, game)?;
    assert_eq!(now_histry_2, [1, 1, 30, 0, 1, 0]); // bet call deal check bet fold.

    let valid_actions_history = gr.call_get_valid_actions_history(&mut store, game)?;
    let status_index = [
        OopAction, IpAction, Chance, OopAction, IpAction, OopAction, Terminal,
    ];
    for (round_idx, actions_detail) in valid_actions_history.iter().enumerate() {
        // println!("Round {}: {:?}", round_idx + 1, actions_detail);
        assert_eq!(actions_detail.game_status, status_index[round_idx]);
    }

    Ok(())
}
