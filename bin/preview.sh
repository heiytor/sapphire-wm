#!/bin/sh

set -e

XEPHYR=$(whereis -b Xephyr | cut -f2 -d' ')
DIR=$(dirname "$(readlink -f "$0")")

export RUST_LOG="trace"
cargo build

xinit "$DIR/xinitrc" -- \
    "$XEPHYR" \
        :100 \
        -ac \
        -screen 1380x720\
        -host-cursor
