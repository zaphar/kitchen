{pkgs? (import <nixpkgs>) {},
 version,
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
    # TODO(jwall): Use the makefile for as much of this as possible.
    buildPhase = ''
        mkdir -p $out
        cd web
        cp -r static $out
		export project=kitchen
		sh ../scripts/wasm-build.sh release
		sh ../scripts/wasm-opt.sh release
        rm -f $out/kitchen_wasm_bg.wasm
        cp -r index.html $out
        cp -r favicon.ico $out
        rm -rf $out/release
        rm -rf $out/wasm32-unknown-unknown
    '';
}
