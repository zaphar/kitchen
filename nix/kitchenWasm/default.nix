{pkgs? (import <nixpkgs>) {},
 version ? "0.2.1",
 cargoVendorDeps ? (import ./../cargoVendorDeps/default.nix {inherit pkgs version; }),
 trunk ? (import ./../trunk/default.nix {inherit pkgs;}),
}:
with pkgs;
let
    pname = "kitchen-wasm";
    src = ./../..;
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
    # TODO(jwall): Build this from the root rather than the src.
    buildPhase = ''
        trunk build --release --public-url /ui/ --dist ./dist web/index.html || echo ignoring staging errors for now;
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