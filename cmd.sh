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
*)
  echo "please input a cmd (bench)"
  ;;
esac