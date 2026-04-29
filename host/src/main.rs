use anyhow::Result;
use clap::{Parser, ValueEnum};
use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

bindgen!({
    path: "../wit",
    world: "string-processor",
});

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Guest {
    Rust,
    Js,
    Python,
}

impl Guest {
    fn component_path(self) -> &'static str {
        match self {
            Guest::Rust => concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../target/wasm32-wasip2/release/guest_rust.wasm"
            ),
            Guest::Js => concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../guests/js/dist/guest_js.wasm"
            ),
            Guest::Python => concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../guests/python/dist/guest_python.wasm"
            ),
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, value_enum, default_value_t = Guest::Rust)]
    guest: Guest,

    input: Option<String>,
}

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
    let args = Args::parse();

    let mut config = Config::new();
    config.cache_config_load_default()?;
    let engine = Engine::new(&config)?;
    let component = Component::from_file(&engine, args.guest.component_path())?;

    let mut linker: Linker<State> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;

    let state = State {
        ctx: WasiCtxBuilder::new().inherit_stdio().build(),
        table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);

    let processor = StringProcessor::instantiate(&mut store, &component, &linker)?;

    let input = args.input.unwrap_or_else(|| "rust wasm".to_string());
    let output = processor.call_process_string(&mut store, &input)?;
    println!("{output}");
    Ok(())
}
