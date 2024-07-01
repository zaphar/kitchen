{pkgs? (import <nixpkgs>) {},
 version,
 features ? "",
 rust-wasm,
 wasm-bindgen,
 lockFile,
 outputHashes,
}:
with pkgs;
let
    pname = "kitchen-wasm";
    src = ./../..;
    # NOTE(jwall): Because we use wasm-pack directly below we need
    # the cargo dependencies to already be installed.
     cargoDeps = (pkgs.rustPlatform.importCargoLock { inherit lockFile outputHashes; });
in
# TODO(zaphar): I should actually be leveraging naersklib.buildPackage with a postInstall for the optimization and bindgen
stdenv.mkDerivation {
    inherit src pname;
    version = version;
    # we need wasmb-bindgen v0.2.83 exactly
    buildInputs = [ rust-wasm wasm-bindgen wasm-pack binaryen];
    propagatedBuildInputs = [ rust-wasm wasm-bindgen wasm-pack binaryen];
    phases = [ "postUnpackPhase" "buildPhase"];
    postUnpackPhase = ''
        ln -s ${cargoDeps} ./cargo-vendor-dir
        cp -r ./cargo-vendor-dir/.cargo ./
        cp -r $src/* ./
    '';
    # TODO(jwall): Build this from the root rather than the src.
    buildPhase = ''
        mkdir -p $out
        cd web
        cp -r static $out
        cargo build --lib --release --target wasm32-unknown-unknown --target-dir $out ${features} --offline
        wasm-bindgen $out/wasm32-unknown-unknown/release/kitchen_wasm.wasm --out-dir $out --typescript --target web
        wasm-opt $out/kitchen_wasm_bg.wasm -o $out/kitchen_wasm_bg-opt.wasm -O
        rm -f $out/kitchen_wasm_bg.wasm
        mv $out/kitchen_wasm_bg-opt.wasm $out/kitchen_wasm_bg.wasm
        cp -r index.html $out
        cp -r favicon.ico $out
        rm -rf $out/release
        rm -rf $out/wasm32-unknown-unknown
    '';
}
