use anyhow::Result;
use cargo_metadata::MetadataCommand;
use heck::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};
use wit_component::ComponentEncoder;

fn main() -> Result<()> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    println!("cargo::rerun-if-changed=../src");

    let status = Command::new("cargo")
        .arg("build")
        .arg("--package=test-programs")
        .arg("--target=wasm32-wasip1")
        .env("CARGO_TARGET_DIR", &out_dir)
        .env("CARGO_PROFILE_DEV_DEBUG", "1")
        .status()?;
    assert!(status.success());

    let meta = MetadataCommand::new().exec()?;
    let targets = meta
        .packages
        .iter()
        .find(|p| p.name == "test-programs")
        .unwrap()
        .targets
        .iter()
        .filter(|t| t.kind == ["bin"])
        .map(|t| &t.name)
        .collect::<Vec<_>>();

    let mut generated_code = String::new();

    for target in targets {
        let camel = target.to_shouty_snake_case();
        let wasm = out_dir
            .join("wasm32-wasip1")
            .join("debug")
            .join(format!("{target}.wasm"));

        let adapter = match target.as_str() {
            s if s.starts_with("client_") => {
                include_bytes!("wasi_snapshot_preview1.command.wasm").to_vec()
            }
            s if s.starts_with("server_") => {
                include_bytes!("wasi_snapshot_preview1.proxy.wasm").to_vec()
            }
            other => panic!("unknown type {other}"),
        };

        let path = compile_component(&wasm, &adapter)?;
        generated_code += &format!("pub const {camel}_COMPONENT: &str = {path:?};\n");
    }

    fs::write(out_dir.join("gen.rs"), generated_code)?;

    Ok(())
}

// Compile a component, return the path of the binary
fn compile_component(wasm: &Path, adapter: &[u8]) -> Result<PathBuf> {
    let module = fs::read(wasm)?;
    let component = ComponentEncoder::default()
        .module(module.as_slice())?
        .validate(true)
        .adapter("wasi_snapshot_preview1", adapter)?
        .encode()?;
    let out_dir = wasm.parent().unwrap();
    let stem = wasm.file_stem().unwrap().to_str().unwrap();
    let component_path = out_dir.join(format!("{stem}.component.wasm"));
    fs::write(&component_path, component)?;
    Ok(component_path)
}
