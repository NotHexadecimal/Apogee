#!/bin/sh

pushd $(dirname "$0")/physics
wasm-pack build
popd
