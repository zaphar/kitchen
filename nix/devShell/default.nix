{ pkgs, rust-wasm }:
with pkgs;
mkShell {
    buildInputs = (lib.darwin-sdk pkgs) ++ (with pkgs; [wasm-bindgen-cli wasm-pack llvm clang rust-wasm]);
}