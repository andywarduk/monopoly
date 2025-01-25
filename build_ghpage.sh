#!/bin/sh

CLICOLOR_FORCE=1 monopoly-wasm/build.sh || exit 1

rm -rf .github/ghpage || exit 1

cp -r monopoly-wasm/html .github/ghpage || exit 1
