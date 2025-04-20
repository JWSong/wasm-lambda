# register a function
cargo run -- deploy --name echo --file ./target/wasm32-wasip1/release/echo_function.wasm --trigger hello.world

# list registered functions
cargo run -- list

# invoke a function
cargo run -- invoke --id <func_id> --subject hello.world --data "Hello WASM!"
