#!/bin/bash

set -u

wsm="./target/wasm32-wasip1/release-wasi/ints0term2ascii2ipc.wasm"

(
  printf '\0\0\0\0'
  printf '\0\0\0\1'
  printf '\0\0\0\2'

  printf 0123
  printf 4567
  printf 89ab
  printf cdef

  printf helo
  printf wrld
  printf 'hw\0\0'
  printf fuji
  printf taka

) |
  wasmtime run "${wsm}" |
  arrow-cat
