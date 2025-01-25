#!/bin/sh

CLICOLOR_FORCE=1 cargo run --bin monopoly-tui --release -- "$@"
