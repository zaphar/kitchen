{pkgs? (import <nixpkgs>) {},
 version ? "0.2.1",
 rust-wasm,
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
            "sycamore-0.8.2" = "sha256-I+NTfT83l8kST//IxJOZmeuhi1xWX070LToWPRU9j2A=";
            "sqlx-0.6.2" = "sha256-X/LFvtzRfiOIEZJiVzmFvvULPpjhqvI99pSwH7a//GM=";
        };
     });
in
stdenv.mkDerivation {
    inherit src pname;
    version = version;
    # we need wasmb-bindgen v0.2.81 exactly
    buildInputs = [ rust-wasm wasm-bindgen-cli wasm-pack binaryen];
    propagatedBuildInputs = [ rust-wasm wasm-bindgen-cli wasm-pack binaryen];
    phases = [ "postUnpackPhase" "buildPhase"];
    postUnpackPhase = ''
        ln -s ${cargoDeps} ./cargo-vendor-dir
        cp -r ./cargo-vendor-dir/.cargo ./
        cp -r $src/* ./
    '';
    # TODO(jwall): Build this from the root rather than the src.
    buildPhase = ''
        echo building with wasm-pack
        mkdir -p $out
        cd web
        cp -r static $out
        RUST_LOG=info wasm-pack build --mode no-install --release --target web --out-dir $out;
        cp -r index.html $out
    '';
}