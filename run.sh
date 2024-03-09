cargo build --release
rm -rf ~/workspace/projects/argnmp.github.io/* 
./target/release/helium --config config.yaml
