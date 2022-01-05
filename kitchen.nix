{nixpkgs, gitignoreSrc}:
with nixpkgs;
    rustPlatform.buildRustPackage {
        pname = "kitchen";
        version = "0.1.0";
        src = gitignoreSrc.gitignoreSource ./.;
        cargoSha256 = "sha256-SCTyR2TN6gNRkDeJOPPJQ2vJg9ClkLx0RJuMLpUWYBY=";
    }