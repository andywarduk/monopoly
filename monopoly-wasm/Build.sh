#!/bin/bash

wasm-pack build --target web

if [ $? -eq 0 ]
then
	python3 -m http.server	
fi
