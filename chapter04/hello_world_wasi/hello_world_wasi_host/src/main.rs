use wasmtime_wasi::{WasiCtxView, WasiView};

wasmtime::component::bindgen!({
 path: "../hello_world_wasi_guest/",
 world: "example",
});

struct State {
    wasi: wasmtime_wasi::WasiCtx,
    table: wasmtime_wasi::ResourceTable,
}

impl WasiView for State {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

fn main() {
    println!("start");
    let mut config = wasmtime::Config::default();
    config.wasm_component_model(true);
    let engine = wasmtime::Engine::new(&config).unwrap();
    let mut linker = wasmtime::component::Linker::<State>::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).unwrap();
    let wasi = wasmtime_wasi::WasiCtxBuilder::new()
        .inherit_stdout()
        .build();
    println!("wasi built");
    let mut store = wasmtime::Store::new(
        &engine,
        State {
            wasi,
            table: wasmtime_wasi::ResourceTable::new(),
        },
    );
    println!("store built");
    let component =
        wasmtime::component::Component::from_file(&engine, "../hello_world_wasi_guest/greet.wasm")
            .unwrap();
    println!("component built");
    let app = Example::instantiate(&mut store, &component, &linker).unwrap();
    println!("app instantiated");

    (0..50).for_each(|i| {
        app.call_greet(&mut store, &format!("World {i}")).unwrap();
    });
}
