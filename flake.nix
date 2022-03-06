{
    description = "kitchen";
    # Pin nixpkgs
    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/adf7f03d3bfceaba64788e1e846191025283b60d";
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
            trunkGen = (import ./nix/trunk/default.nix);
            kitchenWasmGen = (import ./nix/kitchenWasm/default.nix);
            cargoVendorGen = (import ./nix/cargoVendorDeps/default.nix);
            moduleGen = (import ./nix/kitchen/module.nix);
            version = "0.2.1";
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
                trunk = trunkGen { inherit pkgs naersk-lib; };
                cargoVendorDeps = cargoVendorGen {
                    inherit pkgs version;
                    lockFile = ./Cargo.lock;
                }; 
                kitchenWasm = kitchenWasmGen {
                    inherit pkgs cargoVendorDeps rust-wasm trunk version;
                };
                kitchen = (kitchenGen {
                    inherit pkgs version naersk-lib kitchenWasm;# cargoVendorDeps;
                    # Because it's a workspace we need the other crates available as source
                    root = (pkgs.callPackage gitignore { }).gitignoreSource ./.;
                });
                module = moduleGen {inherit kitchen;};
            in
            {
                packages = {
                    inherit trunk
                            cargoVendorDeps
                            kitchenWasm
                            kitchen
                            ;
                };
                defaultPackage = kitchen;
                nixosModules.kitchen = module;
                defaultApp = {
                    type = "app";
                    program = "${kitchen}/bin/kitchen";
                };
            } 
        );
}