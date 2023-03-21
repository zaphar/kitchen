let
    lib = import ../lib/lib.nix;
in
{ pkgs, rust-wasm, wasm-pack-hermetic }:
with pkgs;
mkShell {
    buildInputs = (lib.darwin-sdk pkgs) ++ (with pkgs; [wasm-bindgen-cli wasm-pack-hermetic llvm clang rust-wasm]);
}