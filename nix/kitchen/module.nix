{kitchen}:
{config, lib, pkgs, ...}:
with lib;
{
    options = {
        services.kitchen.enable = mkEnableOption "Activates the kitchen recipe/shopping service";
        
        services.kitchen.listenSocket = mkOption {
            description = "Listen socket for the kitchen service";
            default = "0.0.0.0:9003";
            defaultText = "0.0.0.0:9003";
        };
    };
    
    config = mkIf config.services.kitchen.enable {
        nixpkgs.overlays = [
            (final: prev: {
                recipes = (import ../packages/recipes/package.nix) { inherit pkgs; };
            })
        ];
        systemd.services.kitchen = {
            wantedBy = [ "multi-user.target" "default.target" ];
            wants = [ "network.target" ];
            after = [ "networ-online.target" ];
            serviceConfig = {
                restart = "on-failure";
                restartSec = "10s";
                ExecStart = "${kitchen}/bin/kitchen serve --listen ${config.services.kitchen.listenSocket} --dir ${pkgs.recipes}";
            };
        };
    };
}