#!/bin/bash

wasm-pack build --target web

if [ $? -ne 0 ]
then
	exit
fi

cargo build --manifest-path package/Cargo.toml --release

if [ $? -ne 0 ]
then
	exit
fi

./package/target/release/package template/index.htm html

python3 -m http.server 8000 --bind 127.0.0.1 -d html

