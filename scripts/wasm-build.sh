set -x
buildtype=$1;

mkdir -p $out

if [ ${buildtype} = "release" ]; then
	buildtype_flag="--release"
fi

cargo build --lib ${buildtype_flag} --target wasm32-unknown-unknown --target-dir $out --features debug_logs
wasm-bindgen $out/wasm32-unknown-unknown/${buildtype}/${project}_wasm.wasm --out-dir $out --typescript --target web
