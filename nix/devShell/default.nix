{ pkgs, rust-wasm }:
with pkgs;
mkShell {
    buildInputs = (if stdenv.isDarwin then [ pkgs.darwin.apple_sdk.frameworks.Security ] else [ ]) ++ (with pkgs; [wasm-bindgen-cli wasm-pack llvm clang rust-wasm]);
    #buildInputs = with pkgs; [wasm-bindgen-cli wasm-pack];
}