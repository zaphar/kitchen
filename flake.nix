{
    description = "kitchen";
    # Pin nixpkgs
    inputs.nixpkgs = "github:NixOS/nixpkgs/adf7f03d3bfceaba64788e1e846191025283b60d";
    
    inputs.gitignore = { url = "github:hercules-ci/gitignore.nix"; flake = false; };

    inputs.flake-utils.url = "github:numtide/flake-utils";

    outputs = {self, nixpkgs, flake-utils, gitignore}:
        let
            kitchenGen = import ./kitchen.nix;
        in
        flake-utils.lib.eachDefaultSystem (system:
            let pkgs = import nixpkgs { inherit system; }; in
            {
                defaultPackage = (kitchenGen {
                    nixpkgs = pkgs;
                    gitignoreSrc = pkgs.callPackage gitignore { };
                });
            } 
        );
}