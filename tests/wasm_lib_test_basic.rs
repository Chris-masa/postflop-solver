use anyhow::{anyhow, Result};
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
fn game_manager_api_roundtrip() -> Result<()> {
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
    let component =
        Component::from_file(&engine, "target/wasm32-wasip2/release/postflop_solver.wasm")?;

    // instantiate（※あなたの環境では Host_ が返る）
    let world = Host_::instantiate(&mut store, &component, &linker)?;

    // ✅ ここがポイント：
    // export された interface は `world.<package>_<interface>()` みたいなメソッドとして生えます
    // （メソッド名が微妙に違う場合は、コンパイルエラーに候補が出るのでそれに合わせてください）
    let gm = world.holdem_solver_host_game_manager();

    // resource の投影を取得
    let gr = gm.game_resource();

    // WIT: `new: static func(...) -> game-resource;`
    // → wasmtime の生成では `call_new` になります
    let game = gr.call_new(&mut store, "AsKsQs", 0)?;

    // get-game-status: func() -> wit-game-status
    let status = gr.call_get_game_status(&mut store, game)?;
    assert_eq!(
        status,
        exports::holdem_solver::host::game_manager::WitGameStatus::OopAction
    );

    // OOP Action 1
    let check: bool = gr.call_action(&mut store, game, 0).unwrap().unwrap();
    assert!(check);
    let status = gr.call_get_game_status(&mut store, game)?;
    assert_eq!(
        status,
        exports::holdem_solver::host::game_manager::WitGameStatus::IpAction
    );

    // action履歴取得
    let mut action_history_string = gr
        .call_get_history(&mut store, game)?
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    println!("Action History:{}", action_history_string);

    // IP Action 1
    let check: bool = gr.call_action(&mut store, game, 0).unwrap().unwrap();
    assert!(check);
    let status = gr.call_get_game_status(&mut store, game)?;
    assert_eq!(
        status,
        exports::holdem_solver::host::game_manager::WitGameStatus::Chance
    );

    action_history_string = gr
        .call_get_history(&mut store, game)?
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    println!("Action History:{}", action_history_string);

    println!("point 3");

    // card-deal: func(...) -> result<bool, string>
    // カード作成
    let card_index: u32 = gr
        .call_get_card_index_from_str(&mut store, game, "2d")?
        .unwrap();
    println!("card index: {}", card_index); // 2d は index 1
    assert!(card_index == 1); // 2d は index 1

    println!("point 4");

    action_history_string = gr
        .call_get_history(&mut store, game)?
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    println!("Action History:{}", action_history_string);

    let mut board_cards: Vec<String> = gr.call_get_board_cards(&mut store, game)?;
    let mut board_cards_string = board_cards.join(", ");
    println!("{}", board_cards_string);
    assert!(board_cards[2] == "Qs".to_string());

    let dealt: bool = gr.call_action(&mut store, game, card_index)?.unwrap();
    assert!(dealt);

    println!("point 5");

    board_cards = gr.call_get_board_cards(&mut store, game)?;
    board_cards_string = board_cards.join(", ");
    println!("{}", board_cards_string);
    assert!(board_cards[3] == "2d".to_string());

    println!("point 6");

    // 終わったら resource を明示的に drop（wasmtime の ResourceAny 仕様）
    game.resource_drop(&mut store)?;

    Ok(())
}
