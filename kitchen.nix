{nixpkgs, gitignoreSrc}:
with nixpkgs;
    rustPlatform.buildRustPackage {
        pname = "kitchen";
        version = "0.0.2";
        src = gitignoreSrc.gitignoreSource ./.;
        cargoSha256 = "sha256-DmUWZbZL8A5ht9ujx70qDvT6UC1CKiY6LtwWmKMvVhs=";
    }