let
    lib = import ../lib/lib.nix;
in
{ pkgs, rust-wasm, wasm-pack-hermetic, wasm-bindgen, cargo-wasm2map }:
with pkgs;
mkShell {
    buildInputs = (lib.darwin-sdk pkgs) ++ (with pkgs; [wasm-bindgen wasm-pack-hermetic llvm clang rust-wasm binaryen cargo-wasm2map]);
}
