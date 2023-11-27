let
  my-lib = import ../lib/lib.nix;
in
{pkgs,
 naersk-lib,
 rust-wasm,
}:
with pkgs;
(naersk-lib.buildPackage rec {
    pname = "wasm-pack";
    version = "v0.12.1";
    buildInputs = [ rust-wasm pkgs.openssl curl];
    nativeBuildInputs = (my-lib.darwin-sdk pkgs) ++ [llvm clang pkg-config];
    OPENSSL_NO_VENDOR=1;
    # The checks use network so disable them here
    doCheck = false;
    src = fetchFromGitHub {
      owner = "rustwasm";
      repo = "wasm-pack";
      rev = version;
  	  hash = "sha256-L4mCgUPG4cgTUpCoaIUOTONBOggXn5vMyPKj48B3MMk=";
    };
    cargoBuildOptions = opts: opts ++ ["-p" "${pname}" ];
})
