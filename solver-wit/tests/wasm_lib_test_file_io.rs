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
fn game_manager_io_test() -> Result<()> {
    // Component model を有効化（明示しておくと事故が減ります）
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

    // ↑ Resource準備
    // ↓ ここからが個別テスト
    let mut compressed: Vec<u8>;
    {
        let game_origin = gr.call_new(&mut store, "AsKsQs", 0)?;
        compressed = gr
            .call_get_compressed_result(&mut store, game_origin)
            .unwrap()
            .unwrap();
        // origin_gameはスコープ外で破棄
    }

    let game_loaded = gr
        .call_from_cache(&mut store, &mut compressed)
        .unwrap()
        .unwrap();
    let status = gr.call_get_game_status(&mut store, game_loaded)?;
    assert_eq!(status, OopAction);

    // OOP Action 1
    let check: bool = gr.call_action(&mut store, game_loaded, 0).unwrap().unwrap();
    assert!(check);
    let status = gr.call_get_game_status(&mut store, game_loaded)?;
    assert_eq!(status, IpAction);

    // action履歴取得
    let mut action_history_string = gr
        .call_get_history(&mut store, game_loaded)?
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    println!("Action History:{}", action_history_string);

    // IP Action 1
    let check: bool = gr.call_action(&mut store, game_loaded, 0).unwrap().unwrap();
    assert!(check);
    let status = gr.call_get_game_status(&mut store, game_loaded)?;
    assert_eq!(status, Chance);
    Ok(())
}
