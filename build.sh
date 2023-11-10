cd ./render
./build.sh
cp ./dist/* ../template/static/dist

cd ../template
./build.sh

cd ../
rm -rf /Users/tyler/workspace/projects/argnmp.github.io/*

pwd
cargo run -- --config config.yaml
