use anyhow::{Context, Result};
use std::path::PathBuf;
use wasmtime::{Config, Engine};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: precompile <input.wasm> <output.cwasm>");
        std::process::exit(2);
    }
    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);

    let mut config = Config::new();
    config.cache_config_load_default()?;
    let engine = Engine::new(&config)?;

    let wasm = std::fs::read(&input)
        .with_context(|| format!("failed to read {}", input.display()))?;
    let serialized = engine.precompile_component(&wasm)?;
    std::fs::write(&output, &serialized)
        .with_context(|| format!("failed to write {}", output.display()))?;

    println!(
        "wrote {} ({} bytes)",
        output.display(),
        serialized.len()
    );
    Ok(())
}
