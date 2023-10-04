#!/bin/bash

cmd='./cmd.sh [cmd]
    bench    run example bench'

case $1 in
bench)
  cd ./example || echo "\n not found path:./example" && exit 1
  cargo bench -- --verbose
  ;;
test)
  cargo test --bin example --verbose
  ;;
build_wasm)
  cd ./example/wasm_example
  cargo build --target wasm32-unknown-unknown --release
  ls ../../target/wasm32-unknown-unknown/release
  ;;
*)
  echo "please input a cmd (bench)"
  ;;
esac