use wasmtime::component::{bindgen, Component, Linker};
use wasmtime::{Config, Engine, Store};

bindgen!({
    world: "host",
    path: "./wit/host.wit", // プロジェクトのルートからのパス
});

fn main() {
    // Set the path to your wasm file here for testing
    let wasm_file = "./target/wasm32-unknown-unknown/debug/postflop_solver.wasm";
    println!("Using wasm file: {}", wasm_file);
    println!("point 1");
    // コンポーネントの有効化
    println!("point 2");
    let engine = Engine::default();
    println!("point 3");
    let mut linker = Linker::new(&engine);
    // add_to_linker_sync(&mut linker);
    println!("point 4");
    let mut store = Store::new(&engine, ());
    println!("point 5");
    let component = Component::from_file(&engine, wasm_file).unwrap();
    println!("point 6");
    let provider = Host_::instantiate(&mut store, &component, &linker).unwrap();
    println!("point 7");
    let wit_module = provider.holdem_solver_host_my_host();
    let now = wit_module.call_get_date(&mut store).unwrap();
    println!("point 8");
    // let now = wasm_component.call_get_date(&mut store).unwrap();
    println!("Current date and time: {}", now);
}
