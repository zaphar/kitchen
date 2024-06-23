{pkgs? (import <nixpkgs>) {},
 version,
 features ? "",
 rust-wasm,
 wasm-bindgen,
}:
with pkgs;
let
    pname = "kitchen-wasm";
    src = ./../..;
    lockFile = ./../../Cargo.lock;
    # NOTE(jwall): Because we use wasm-pack directly below we need
    # the cargo dependencies to already be installed.
     cargoDeps = (pkgs.rustPlatform.importCargoLock { inherit lockFile; outputHashes = {
            # I'm maintaining some patches for these so the lockfile hashes are a little
            # incorrect. We override those here.
            "wasm-web-component-0.2.0" = "sha256-quuPgzGb2F96blHmD3BAUjsWQYbSyJGZl27PVrwL92k=";
            "sycamore-0.8.2" = "sha256-D968+8C5EelGGmot9/LkAlULZOf/Cr+1WYXRCMwb1nQ=";
            "sqlx-0.6.2" = "sha256-X/LFvtzRfiOIEZJiVzmFvvULPpjhqvI99pSwH7a//GM=";
        };
     });
in
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
        set -x
        echo building with wasm-pack
        wasm-pack --version
        mkdir -p $out
        cd web
        cp -r static $out
        cargo build --lib --release --target wasm32-unknown-unknown --target-dir $out --offline
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
