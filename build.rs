use std::process::Command;
use std::path::Path;

fn main(){
    println!("cargo:warning=wasm build start");
    Command::new("wasm-pack")
        .args(&["build", "--out-dir", "dist", "--target", "web", "--no-typescript", "--no-pack"])
        .current_dir(&Path::new("./render"))
        .status()
        .unwrap();
    println!("cargo:warning=wasm build end");
    println!("cargo:warning=template build start");
    Command::new("npm")
        .args(&["run", "build"])
        .current_dir(&Path::new("./template"))
        .status()
        .unwrap();
    println!("cargo:warning=template build end");
   
}
