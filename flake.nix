{
    description = "kitchen";
    # Pin nixpkgs
    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs";
        gitignore = { url = "github:hercules-ci/gitignore.nix"; flake = false; };
        flake-utils.url = "github:numtide/flake-utils";
        rust-overlay = {
          url = "github:oxalica/rust-overlay?ref=stable";
          inputs.nixpkgs.follows = "nixpkgs";
        };
        naersk.url = "github:nix-community/naersk";
        flake-compat = { url = github:edolstra/flake-compat; flake = false; };
    };
    outputs = {self, nixpkgs, flake-utils, rust-overlay, naersk, gitignore, flake-compat}:
        let
            kitchenGen = (import ./nix/kitchen/default.nix);
            kitchenWasmGen = (import ./nix/kitchenWasm/default.nix);
            moduleGen = (import ./nix/kitchen/module.nix);
            version = "0.2.18";
        in
        flake-utils.lib.eachDefaultSystem (system:
            let
                overlays = [ rust-overlay.overlays.default ];
                pkgs = import nixpkgs { inherit system overlays; };
                rust-wasm = pkgs.rust-bin.stable."1.64.0".default.override {
                  extensions = [ "rust-src" ];
                  # Add wasm32 as an extra target besides the native target.
                  targets = [ "wasm32-unknown-unknown" ];
                };
                # make sure to use our rust-wasm build target as the rust toolchain
                # in naersk.
                naersk-lib = pkgs.callPackage naersk {
                    rustc = rust-wasm;
                    cargo = rust-wasm;
                };
                kitchenWasm = kitchenWasmGen {
                    inherit pkgs rust-wasm version;
                };
                kitchen = (kitchenGen {
                    inherit pkgs version naersk-lib kitchenWasm rust-wasm;
                    # Because it's a workspace we need the other crates available as source
                    # TODO(jwall): gitignoreSource is broken right now due to being impure.
                    #root = (pkgs.callPackage gitignore { }).gitignoreSource ./.;
                    root = ./.;
                });
                kitchenWasmDebug = kitchenWasmGen {
                    inherit pkgs rust-wasm version;
                    features = "--features debug_logs";
                };
                kitchenDebug = (kitchenGen {
                    inherit pkgs version naersk-lib rust-wasm;
                    kitchenWasm = kitchenWasmDebug;
                    # Because it's a workspace we need the other crates available as source
                    # TODO(jwall): gitignoreSource is broken right now due to being impure.
                    #root = (pkgs.callPackage gitignore { }).gitignoreSource ./.;
                    root = ./.;
                });
                module = moduleGen {inherit kitchen;};
            in
            {
                packages = {
                    inherit kitchenWasm
                            kitchen
                            kitchenWasmDebug
                            kitchenDebug
                            ;
                };
                defaultPackage = kitchen;
                nixosModules.kitchen = module;
                defaultApp = {
                    type = "app";
                    program = "${kitchen}/bin/kitchen";
                };
                devShell = pkgs.callPackage ./nix/devShell/default.nix { inherit rust-wasm; };
            } 
        );
}