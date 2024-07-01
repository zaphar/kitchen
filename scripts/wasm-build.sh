set -x
buildtype=$1;

if [ -eq ${buildtype} = "release" ]; then
	builtype_flag="--release"
fi

cargo build --lib ${buildtype_flag} --target wasm32-unknown-unknown --target-dir $out --features debug_logs
wasm-bindgen $out/wasm32-unknown-unknown/${buildtype}/kitchen_wasm.wasm --out-dir $out --typescript --target web
