#!/bin/sh
docker run --rm -v ./data:/data ghcr.io/danbugs/serverside-wasm-book-code/wasmtime-onnx:latest run -Snn \
 --dir /fixture::fixture \
 --dir /data::data /data/wasi_nn_example.wasm /data/cat.jpg
