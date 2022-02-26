{pkgs? (import <nixpkgs>) {},
 version ? "0.2.1",
 cargoVendorDeps ? (import ./../cargoVendorDeps/default.nix {inherit pkgs version; }),
 trunk ? (import ./../trunk/default.nix {inherit pkgs;}),
}:
with pkgs;
let
    pname = "kitchen-wasm";
    src = ./../../web;
in
stdenv.mkDerivation {
    inherit src pname;
    version = version;
    buildInputs = [ trunk ];
    phases = [ "postUnpackPhase" "buildPhase" "installPhase" ];
    postUnpackPhase = ''
        ln -s ${cargoVendorDeps} ./cargo-vendor-dir
        cp -r ./cargo-vendor-dir/.cargo ./
        cp -r $src/* ./
    '';
    # NOTE(jwall): For some reason trunk is trying to do something with staging that
    # nix doesn't like. We suppress the message for now but I'd like to
    # know why trunk can't create those directories.
    buildPhase = ''
        trunk build --release --public-url /ui/ --dist ./dist || echo ignoring staging errors for now;
        pwd
        ls -al .
    '';
    installPhase = ''
        pwd
        ls -al .
        mkdir -p $out
        echo cp -r ./dist $out/
        cp -r ./dist $out/
    '';
}