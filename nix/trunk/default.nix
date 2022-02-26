{pkgs ? (import <nixpkgs>) {}}:
with pkgs;
rustPlatform.buildRustPackage rec {
    pname = "trunk";
    version = "v0.14.0";
    src = fetchFromGitHub {
        owner = "thedodd";
        repo = pname;
        rev = version;
        sha256 = "sha256-69MQDIF79pSuaOgZEIqb/ESPQzL7MUiQaJaxPccGxo8=";
    };
    cargoSha256 = "sha256-RPE2wMggD8nXpuUkRsvG5wxIWCHifD4RLLLPkq8HOEQ=";

    # Trunk uses the network in it's test which is lame. We'll work around
    # by disabling here for now.
    doCheck = false;
    meta = {
        description = "Trunk rust web assembly bundler";
    };
}