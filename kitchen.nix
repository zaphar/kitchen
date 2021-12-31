{nixpkgs, gitignoreSrc}:
with nixpkgs;
    rustPlatform.buildRustPackage {
        pname = "kitchen";
        version = "0.0.2";
        src = gitignoreSrc.gitignoreSource ./.;
        cargoSha256 = "sha256-aSw+BX90rmcagVOkLVEfjlqTi+dv4QVT7JPZQd3eKjA=";
    }