{pkgs? (import <nixpkgs>) {},
 version ? "0.2.1",
 cargoVendorDeps ? (import ./../cargoVendorDeps/default.nix {inherit pkgs version; }),
 rust-wasm,
 trunk ? (import ./../trunk/default.nix {inherit pkgs;}),
}:
with pkgs;
let
    pname = "kitchen-wasm";
    src = ./../..;
in
stdenv.mkDerivation {
    inherit src pname;
    version = version;
    # we need wasmb-bindgen v0.2.78 exactly
    buildInputs = [ rust-wasm wasm-bindgen-cli wasm-pack binaryen];
    phases = [ "postUnpackPhase" "buildPhase"];
    postUnpackPhase = ''
        ln -s ${cargoVendorDeps} ./cargo-vendor-dir
        cp -r ./cargo-vendor-dir/.cargo ./
        cp -r $src/* ./
    '';
    # TODO(jwall): Build this from the root rather than the src.
    buildPhase = ''
        echo building with wasm-pack
        mkdir -p $out
        cd web
        cp -r static $out
        wasm-pack build --target web --out-dir $out;
        cp -r index.html $out
    '';
}