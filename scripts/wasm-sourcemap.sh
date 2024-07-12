set -x

cargo-wasm2map wasm2map --patch $out/${project}_wasm_bg.wasm --base-url=http://localhost:3030
