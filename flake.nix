{
    description = "kitchen";
    # Pin nixpkgs
    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/adf7f03d3bfceaba64788e1e846191025283b60d";
        gitignore = { url = "github:hercules-ci/gitignore.nix"; flake = false; };
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs = {self, nixpkgs, flake-utils, gitignore}:
        let
            kitchenGen = (import ./nix/default.nix);
            trunkGen = (import ./nix/trunk/default.nix);
            kitchenWasmGen = (import ./nix/kitchenWasm/default.nix);
            cargoVendorGen = (import ./nix/cargoVendorDeps/default.nix);
            version = "0.2.1";
        in
        flake-utils.lib.eachDefaultSystem (system:
            let
                pkgs = import nixpkgs { inherit system; };
                trunk = trunkGen { inherit pkgs; };
                cargoVendorDeps = cargoVendorGen {
                    inherit pkgs version;
                    lockFile = ./Cargo.lock;
                }; 
                kitchenWasm = kitchenWasmGen {
                    inherit pkgs cargoVendorDeps trunk version;
                };
                #kitchen = (kitchenGen {
                #    inherit pkgs cargoDeps version kitchenWasm;
                ##    gitignoreSrc = nixpkgs.callPackage gitignore { };
                #});
            in
            {
                packages = {
                    inherit trunk
                            cargoVendorDeps
                            kitchenWasm
                            # kitchen
                            ;
                };
                defaultPackage = cargoVendorDeps;
            } 
        );
}