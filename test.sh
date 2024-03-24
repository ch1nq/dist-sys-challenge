#!/bin/bash
cargo build --release --bin "$1" && maelstrom/maelstrom test -w "$1" --bin "target/release/$1"