let
    lib = import ../lib/lib.nix;
in
{ pkgs, rust-wasm, wasm-pack }:
with pkgs;
mkShell {
    buildInputs = (lib.darwin-sdk pkgs) ++ (with pkgs; [wasm-bindgen-cli wasm-pack llvm clang rust-wasm]);
}