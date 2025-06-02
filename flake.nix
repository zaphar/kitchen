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
       flake-compat = { url = "github:edolstra/flake-compat"; flake = false; };
		   cargo-wasm2map-src = { url = "github:mtolmacs/wasm2map"; flake = false; };
    };
    outputs = {nixpkgs, flake-utils, rust-overlay, naersk, cargo-wasm2map-src, ...}:
        let
            kitchenGen = (import ./nix/kitchen/default.nix);
            kitchenWasmGen = (import ./nix/kitchenWasm/default.nix);
            moduleGen = (import ./nix/kitchen/module.nix);
            wasm-packGen = (import ./nix/wasm-pack/default.nix);
            wasm-bindgenGen = (import ./nix/wasm-bindgen/default.nix);
            version = "0.2.25";
        in
        flake-utils.lib.eachDefaultSystem (system:
            let
                overlays = [ rust-overlay.overlays.default ];
                pkgs = import nixpkgs { inherit system overlays; };
                rust-wasm = pkgs.rust-bin.stable."1.87.0".default.override {
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
                # TODO(jwall): Do the same thing for wasm-bindgen as well?
                # We've run into a few problems with the bundled wasm-pack in nixpkgs.
                # Better to just control this part of our toolchain directly.
                wasm-pack = wasm-packGen {
                    inherit rust-wasm naersk-lib pkgs;
                };
				cargo-wasm2map = naersk-lib.buildPackage {
					pname = "cargo-wasm2map";
					version = "v0.1.0";
					build-inputs = [ rust-wasm ];
					src = cargo-wasm2map-src;
					cargoBuildOptions = opts: opts ++ ["-p" "cargo-wasm2map" ];
				};
                wasm-bindgen = pkgs.callPackage wasm-bindgenGen { inherit pkgs; };
                kitchenWasm = kitchenWasmGen {
                    inherit pkgs rust-wasm wasm-bindgen version cargo-wasm2map;
                    lockFile = ./Cargo.lock;
                    outputHashes = {
                        # I'm maintaining some patches for these so the lockfile hashes are a little
                        # incorrect. We override those here.
                        "wasm-web-component-0.2.0" = "sha256-quuPgzGb2F96blHmD3BAUjsWQYbSyJGZl27PVrwL92k=";
                        "sycamore-0.8.2" = "sha256-D968+8C5EelGGmot9/LkAlULZOf/Cr+1WYXRCMwb1nQ=";
                    };
                };
                kitchen = (kitchenGen {
                    inherit pkgs version naersk-lib kitchenWasm rust-wasm;
                    # Because it's a workspace we need the other crates available as source
                    # TODO(jwall): gitignoreSource is broken right now due to being impure.
                    #root = (pkgs.callPackage gitignore { }).gitignoreSource ./.;
                    root = ./.;
                });
                kitchenWasmDebug = kitchenWasmGen {
                    inherit pkgs rust-wasm wasm-bindgen version cargo-wasm2map;
                    lockFile = ./Cargo.lock;
                    outputHashes = {
                        # I'm maintaining some patches for these so the lockfile hashes are a little
                        # incorrect. We override those here.
                        "wasm-web-component-0.2.0" = "sha256-quuPgzGb2F96blHmD3BAUjsWQYbSyJGZl27PVrwL92k=";
                        "sycamore-0.8.2" = "sha256-D968+8C5EelGGmot9/LkAlULZOf/Cr+1WYXRCMwb1nQ=";
                    };
                    #features = "--features debug_logs";
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
                devShell = pkgs.callPackage ./nix/devShell/default.nix {
                    inherit rust-wasm wasm-bindgen cargo-wasm2map;
                    wasm-pack-hermetic = wasm-pack;
                };
            } 
        );
}
