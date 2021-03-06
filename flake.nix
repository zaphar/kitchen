{
    description = "kitchen";
    # Pin nixpkgs
    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs";
        gitignore = { url = "github:hercules-ci/gitignore.nix"; flake = false; };
        flake-utils.url = "github:numtide/flake-utils";
        rust-overlay = {
          url = "github:oxalica/rust-overlay";
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
            version = "0.2.8";
        in
        flake-utils.lib.eachDefaultSystem (system:
            let
                overlays = [ rust-overlay.overlay ];
                pkgs = import nixpkgs { inherit system overlays; };
                # TODO(jwall): Could this get by with minimal instead?
                rust-wasm = pkgs.rust-bin.stable.latest.default.override {
                  extensions = [ "rust-src" ];
                  targets = [ "wasm32-unknown-unknown" ];
                };
                naersk-lib = naersk.lib."${system}";
                kitchenWasm = kitchenWasmGen {
                    inherit pkgs rust-wasm version;
                };
                kitchen = (kitchenGen {
                    inherit pkgs version naersk-lib kitchenWasm;
                    # Because it's a workspace we need the other crates available as source
                    root = (pkgs.callPackage gitignore { }).gitignoreSource ./.;
                });
                module = moduleGen {inherit kitchen;};
            in
            {
                packages = {
                    inherit kitchenWasm
                            kitchen
                            ;
                };
                defaultPackage = kitchen;
                nixosModules.kitchen = module;
                defaultApp = {
                    type = "app";
                    program = "${kitchen}/bin/kitchen";
                };
                devShell = pkgs.mkShell {
                    buildInputs = [ rust-wasm ] ++ (with pkgs; [wasm-bindgen-cli wasm-pack httplz]);
                };
            } 
        );
}