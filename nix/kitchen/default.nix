{pkgs ? (import <nixpkgs>) {},
 # Because it's a workspace we need the other crates available as source
 root,
 kitchenWasm,
 version,
 naersk-lib,
 rust-wasm,
}:
with pkgs;
(naersk-lib.buildPackage rec {
    pname = "kitchen";
    inherit version;
    buildInputs = [ rust-wasm ];
    # However the crate we are building has it's root in specific crate.
    src = root;
    nativeBuildInputs = if stdenv.isDarwin then [ xcbuild ] else [ ];
    cargoBuildOptions = opts: opts ++ ["-p" "${pname}" ];
    postPatch = ''
      mkdir -p web/dist
      cp -r ${kitchenWasm}/* web/dist/
      ls web/dist/
    '';
})