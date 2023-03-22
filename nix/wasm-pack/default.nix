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
    version = "v0.11.0";
    buildInputs = [ rust-wasm pkgs.openssl curl];
    nativeBuildInputs = (my-lib.darwin-sdk pkgs) ++ [llvm clang pkg-config];
    OPENSSL_NO_VENDOR=1;
    # The checks use network so disable them here
    doCheck = false;
    src = fetchFromGitHub {
      owner = "rustwasm";
      repo = "wasm-pack";
      rev = version;
      sha256 = "sha256-3iwXoYnmrZsbwFUR41uI/4jnCF0OjeRO7UqVDaGJJbQ=";
    };
    cargoBuildOptions = opts: opts ++ ["-p" "${pname}" ];
})