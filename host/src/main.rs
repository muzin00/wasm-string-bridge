use anyhow::Result;
use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

bindgen!({
    path: "../wit",
    world: "string-processor",
});

const COMPONENT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../target/wasm32-wasip2/release/guest_rust.wasm"
);

struct State {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for State {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

fn main() -> Result<()> {
    let engine = Engine::default();
    let component = Component::from_file(&engine, COMPONENT_PATH)?;

    let mut linker: Linker<State> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;

    let state = State {
        ctx: WasiCtxBuilder::new().inherit_stdio().build(),
        table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);

    let processor = StringProcessor::instantiate(&mut store, &component, &linker)?;

    let input = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "rust wasm".to_string());
    let output = processor.call_process_string(&mut store, &input)?;
    println!("{output}");
    Ok(())
}
