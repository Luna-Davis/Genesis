#!/usr/bin/bash

echo "Building..."
cargo build --release
echo "Installing..."
cp target/release/gen ~/.local/bin
