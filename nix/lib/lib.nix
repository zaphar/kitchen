{
  darwin-sdk = pkgs: with pkgs; (if stdenv.isDarwin then (with darwin.apple_sdk.frameworks; [
      xcbuild
      Security
      fixDarwinDylibNames
    ]) else [ ]);
}