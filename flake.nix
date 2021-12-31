{
    description = "kitchen";
    inputs.nixpkgs = {
        type = "indirect";
        id = "nixpkgs";
    };
    
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