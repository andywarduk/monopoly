#!/bin/sh

# Default to building a standalone page and opening it
if [ $# -eq 0 ]
then
	set -- -o -s
fi

CLICOLOR_FORCE=1 monopoly-wasm/build.sh "$@"
