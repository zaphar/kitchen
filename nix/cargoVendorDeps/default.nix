{pkgs ? (import <nixpkgs>) {},
 lockFile ? ./../../Cargo.lock,
 version ? "0.2.1"}:
 let
    cargoDeps = (pkgs.rustPlatform.importCargoLock { inherit lockFile; });
    recipes = ./../../recipes;
in
with pkgs;
stdenv.mkDerivation {
    name = "cargo-vendor-deps";
    version = "0.0.0";
    phases = [ "installPhase" ];
    installPhase = ''
        mkdir -p $out
        cp -r ${cargoDeps}/* $out/
        cp -r ${cargoDeps}/.cargo $out/
        cp -r ${recipes} $out/recipes-${version}
        ls -al $out/
    '';
}
 