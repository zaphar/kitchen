set -x
buildtype=$1;

wasm-opt $out/wasm32-unkown-unkown/${buildtype}/${project}_wasm.wasm --out-dir dist/ -0
rm -f $out/${project}_wasm_bg.wasm
mv $out/${project}_wasm_bg-opt.wasm dist/${project}_wasm_bg.wasm
