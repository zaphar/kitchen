let
  my-lib = import ../lib/lib.nix;
in
{ pkgs
, lib
, rustPlatform
, fetchCrate
, nodejs
, pkg-config
, openssl
, curl
}:

# This package is special so we don't use the naersk infrastructure to build it.
# Instead we crib from the nixpkgs version with some tweaks to work with our
# flake setup.
rustPlatform.buildRustPackage rec {
  pname = "wasm-bindgen-cli";
  # NOTE(jwall): This must exactly match the version of the wasm-bindgen crate
  # we are using.
  version = "0.2.89";

  src = fetchCrate {
    inherit pname version;
    sha256 = "sha256-IPxP68xtNSpwJjV2yNMeepAS0anzGl02hYlSTvPocz8=";
  };

  cargoSha256 = "sha256-pBeQaG6i65uJrJptZQLuIaCb/WCQMhba1Z1OhYqA8Zc=";

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ openssl curl ] ++ (my-lib.darwin-sdk pkgs);

  nativeCheckInputs = [ nodejs ];

  # other tests require it to be ran in the wasm-bindgen monorepo
  cargoTestFlags = [ "--test=reference" ];
}
