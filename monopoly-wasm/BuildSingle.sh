#!/bin/bash

# Run wasm-pack to build the web assembly library and js linkage
wasm-pack build --target web

if [ $? -ne 0 ]
then
	exit
fi

# Run cargo to build the packaging binary
cargo build --manifest-path package/Cargo.toml --release

if [ $? -ne 0 ]
then
	exit
fi

# Package in to the single directory
./package/target/release/package --single template/index.htm single

if [ $? -ne 0 ]
then
	exit
fi

# Print file details
ls -l single/index.htm

# Open the generated file
./LaunchURL.sh single/index.htm
