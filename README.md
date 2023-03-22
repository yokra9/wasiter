# Wasiter

Wasiter performs simple TCP forwarding using [WASI Sockets](https://github.com/WebAssembly/wasi-sockets).

Wasiter は [WASI Sockets](https://github.com/WebAssembly/wasi-sockets) を利用したシンプルな TCP 転送を行います

```bash
# Build
cargo build --target wasm32-wasi --release

# Usage
wasmedge --env REMOTE=127.0.0.1:80 --env LOCAL=127.0.0.1:1234 target/wasm32-wasi/release/wasiter.wasm
curl http://127.0.0.1:1234/index.html

wasmedge --env REMOTE=example.com:80 --env LOCAL=127.0.0.1:1234 target/wasm32-wasi/release/wasiter.wasm
curl -H "Host: example.com" http://127.0.0.1:1234/index.html
```
