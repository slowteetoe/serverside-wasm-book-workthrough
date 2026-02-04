use std::collections::HashMap;

use wasmtime_wasi::{DirPerms, FilePerms, IoView, ResourceTable, WasiCtxBuilder, WasiView};

wasmtime::component::bindgen!({
    path: "./smart_cms.wit",
    world: "app",
});

struct KeyValue {
    mem: std::collections::HashMap<String, String>,
}

impl component::smartcms::kvstore::Host for KeyValue {
    fn get(&mut self, key: String) -> Option<String> {
        self.mem.get(&key).cloned()
    }

    fn set(&mut self, key: String, value: String) -> () {
        self.mem.insert(key, value);
    }
}

struct State {
    key_value: KeyValue,
    wasi: (wasmtime_wasi::WasiCtx, wasmtime_wasi::ResourceTable),
    wasi_nn: wasmtime_wasi_nn::wit::WasiNnCtx,
}

impl WasiView for State {
    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        &mut self.wasi.0
    }
}

impl IoView for State {
    fn table(&mut self) -> &mut wasmtime_wasi::ResourceTable {
        &mut self.wasi.1
    }
}

fn main() {
    let mut config = wasmtime::Config::default();
    config.wasm_component_model(true);

    let engine = wasmtime::Engine::new(&config).unwrap();

    let wasi_table = ResourceTable::new();
    let wasi_ctx = WasiCtxBuilder::new()
        .preopened_dir(".", ".", DirPerms::READ, FilePerms::READ)
        .unwrap()
        .build();
    let (backends, registry) = wasmtime_wasi_nn::preload(&[]).unwrap();
    let wasi_nn_ctx = wasmtime_wasi_nn::wit::WasiNnCtx::new(backends, registry);
    let state = State {
        key_value: KeyValue {
            mem: HashMap::new(),
        },
        wasi: (wasi_ctx, wasi_table),
        wasi_nn: wasi_nn_ctx,
    };
    let mut store = wasmtime::Store::new(&engine, state);
    let component =
        wasmtime::component::Component::from_file(&engine, "guest_with_ml.wasm").unwrap();
    let mut linker = wasmtime::component::Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker).unwrap();
    wasmtime_wasi_nn::wit::add_to_linker(&mut linker, |state: &mut State| {
        wasmtime_wasi_nn::wit::WasiNnView::new(&mut state.wasi.1, &mut state.wasi_nn)
    })
    .unwrap();

    component::smartcms::kvstore::add_to_linker(&mut linker, |state: &mut State| {
        &mut state.key_value
    })
    .unwrap();

    let app = App::instantiate(&mut store, &component, &linker).unwrap();
    println!("{:?}", app.call_run(&mut store).unwrap());
}
