{nixpkgs, gitignoreSrc}:
with nixpkgs;
    nixpkgs.stdenv.mkDerivation rec {
        name = "kitchen";
        src = fetchurl {
            url = "https://github.com/zaphar/kitchen/releases/download/v0.2.0-nix/kitchen-linux";
            sha256 = "1f1lxw893r6afgkhizvhm4pg20qfw3kwf9kbzmkbcw0d21qsd9z2";
        };

        phases = ["installPhase" "patchPhase"];
        installPhase = ''
            mkdir -p $out/bin
            cp $src $out/bin/kitchen
            chmod u+x $out/bin/kitchen
        '';
    }