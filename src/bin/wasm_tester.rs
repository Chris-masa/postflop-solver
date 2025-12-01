use anyhow::Result;
use wasmtime::{component::Component, component::Linker, Engine, Store};
use wasmtime_wasi::preview2::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

// WIT からバインディングを生成
wasmtime::component::bindgen!({
    world: "host",
    path: "wit/host.wit",
});

/// Host 側コンテキスト — WASI + リソーステーブル
struct HostState {
    wasi: WasiCtx,
    table: ResourceTable,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

fn main() -> Result<()> {
    // 1. Wasmtime エンジン
    let engine = Engine::default();

    // 2. コンポーネントとしてビルドされた Wasm をロード
    let component =
        Component::from_file(&engine, "target/wasm32-wasip2/release/postflop_solver.wasm")?;

    // 3. Store に HostState を与える (WASI + ResourceTable)
    let mut store = Store::new(
        &engine,
        HostState {
            wasi: WasiCtxBuilder::new().inherit_stdio().build(),
            table: ResourceTable::new(),
        },
    );

    // 4. Linker を作成
    let mut linker = Linker::<HostState>::new(&engine);

    // 5. WASI Preview 2 を Linker に登録
    wasmtime_wasi::preview2::command::add_to_linker(&mut linker)?;

    // 6. WIT で定義した host バインディングを Linker に登録
    Host::add_to_linker(&mut linker, |state| state)?;

    // 7. コンポーネントを instantiate
    let (bindings, _instance) = Host::instantiate(&mut store, &component, &linker)?;

    // 8. WIT API を呼び出し例
    let gm = bindings.game_manager();
    let game = gm.new(&mut store, "AhKhQd".to_string())?;
    println!("Game instance created: {:?}", game);

    let info = game.get_node_info(&mut store)?;
    println!("Node info: {:?}", info);

    Ok(())
}
