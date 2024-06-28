use std::process::Command;
use std::path::Path;

fn main(){
    println!("cargo::rerun-if-changed=./wasm/src");
    println!("cargo::rerun-if-changed=./template/src");
    println!("cargo::rerun-if-changed=./template/package.json");
    println!("cargo::rerun-if-changed=./template/tailwind.config.js");
    println!("cargo::rerun-if-changed=./template/webpack.config.js");

    println!("cargo:warning=wasm build start");
    Command::new("rm")
        .args(["-rf", "./wasm/dist"])
        .status()
        .expect("rm failed");
    let build_result = Command::new("wasm-pack")
        .args(["build", "--out-dir", "dist/static", "--target", "web", "--no-typescript", "--no-pack"])
        .current_dir(Path::new("./wasm"))
        .status();
    match build_result {
        Ok(exit_status) => {
            if !exit_status.success() {
                println!("cargo:warning=wasm build failed, {}, skip build task. use default build file(./wasm/default)", exit_status);
                return;
            }
            println!("cargo:warning=wasm build end, {}", exit_status);
        },
        Err(err) => {
            println!("cargo:warning=wasm build failed({}), skip build task. use default build file(./wasm/default)", err);
        }
    }

    println!("cargo:warning=template build start");

    let build_result = Command::new("npm")
        .args(["install"])
        .current_dir("./template")
        .status();

    match build_result {
        Ok(exit_status) => {
            if !exit_status.success() {
                println!("cargo:warning=template dependency install failed, {}, skip build task. use default build file(./template/default)", exit_status);
                return;
            }
            println!("cargo:warning=template dependency install end, {}", exit_status);
        },
        Err(err) => {
            println!("cargo:warning=template dependency install failed({}), skip build task. use default build file(./template/default)", err);
            return;
        }
    }

    let build_result = Command::new("npm")
        .args(["run", "build"])
        .current_dir("./template")
        .status();
    match build_result {
        Ok(exit_status) => {
            if !exit_status.success() {
                println!("cargo:warning=template build failed, {}, skip build task. use default build file(./template/default)", exit_status);
                return;
            }
            println!("cargo:warning=template build end, {}", exit_status);
        },
        Err(err) => {
            println!("cargo:warning=template build failed({}), skip build task. use default build file(./template/default)", err);
        }
    }
}
