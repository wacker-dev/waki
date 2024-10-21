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
    println!("cargo::rerun-if-changed=../../waki");

    let wasip2 = version_check::is_min_version("1.82.0").unwrap_or(false);
    let wasi_target = if wasip2 {
        "wasm32-wasip2"
    } else {
        "wasm32-wasip1"
    };

    let status = Command::new("cargo")
        .arg("build")
        .arg("--package=test-programs")
        .arg(format!("--target={wasi_target}"))
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
            .join(wasi_target)
            .join("debug")
            .join(format!("{target}.wasm"));

        let path = if wasip2 {
            wasm
        } else {
            compile_component(
                &wasm,
                match target.as_str() {
                    s if s.starts_with("client_") => {
                        wasi_preview1_component_adapter_provider::WASI_SNAPSHOT_PREVIEW1_COMMAND_ADAPTER
                    }
                    s if s.starts_with("server_") => {
                        wasi_preview1_component_adapter_provider::WASI_SNAPSHOT_PREVIEW1_PROXY_ADAPTER
                    }
                    other => panic!("unknown type {other}"),
                },
            )?
        };
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
