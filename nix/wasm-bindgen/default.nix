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
, stdenv
, curl
, runCommand
}:

# This package is special so we don't use the naersk infrastructure to build it.
# Instead we crib from the nixpkgs version with some tweaks to work with our
# flake setup.
rustPlatform.buildRustPackage rec {
  pname = "wasm-bindgen-cli";
  # NOTE(jwall): This must exactly match the version of the wasm-bindgen crate
  # we are using.
  version = "0.2.84";

  src = fetchCrate {
    inherit pname version;
    sha256 = "sha256-0rK+Yx4/Jy44Fw5VwJ3tG243ZsyOIBBehYU54XP/JGk=";
  };

  cargoSha256 = "sha256-vcpxcRlW1OKoD64owFF6mkxSqmNrvY+y3Ckn5UwEQ50=";

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ openssl curl ] ++ (my-lib.darwin-sdk pkgs);

  nativeCheckInputs = [ nodejs ];

  # other tests require it to be ran in the wasm-bindgen monorepo
  cargoTestFlags = [ "--test=interface-types" ];
}