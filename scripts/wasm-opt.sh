set -x
buildtype=$1;

wasm-opt $out/wasm32-unknown-unknown/${buildtype}/${project}_wasm.wasm --output $out/${project}_wasm_bg-opt.wasm -O
rm -f $out/${project}_wasm_bg.wasm
mv $out/${project}_wasm_bg-opt.wasm $out/${project}_wasm_bg.wasm
