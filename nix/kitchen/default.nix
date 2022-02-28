{pkgs ? (import <nixpkgs>) {},
 # Because it's a workspace we need the other crates available as source
 root,
 kitchenWasm,
 version,
 naersk-lib,
 #cargoVendorDeps ? (import ./../cargoVendorDeps/default.nix {inherit pkgs version; }),
}:
with pkgs;
#let
#  vendorDir = "cargo-vendor-dir";
#  #cargoVendorDrv = cargoVendorDeps;
#in
(naersk-lib.buildPackage rec {
    pname = "kitchen";
    inherit version;
    # However the crate we are building has it's root in specific crate.
    src = root;
    cargoBuildOptions = opts: opts ++ ["-p" "${pname}" ];
    postPatch = ''
      echo ln -s ${kitchenWasm} web/dist
      ln -s ${kitchenWasm} web/dist
    '';
    #  echo cp -r ${cargoVendorDrv}/* ${vendorDir}/
    #  cp -r ${cargoVendorDrv}/* ${vendorDir}/
    #  mkdir -p .cargo
    #  cp -r ${cargoVendorDrv}/.cargo/* .cargo/
})