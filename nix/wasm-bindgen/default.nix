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

  cargoHash = "sha256-EsGFW1f9+E5NnMadP/0rRzFCxVJQb0mlTLz/3zYQ5Ac=";

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ openssl curl ];

  nativeCheckInputs = [ nodejs ];

  # other tests require it to be ran in the wasm-bindgen monorepo
  #cargoTestFlags = [ "--test=reference" ];
  doCheck = false;
}
