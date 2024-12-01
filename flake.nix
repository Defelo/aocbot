{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    ...
  }: let
    inherit (nixpkgs) lib;

    eachDefaultSystem = lib.genAttrs [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];
  in {
    packages = eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      default = pkgs.callPackage ./nix/package.nix {inherit fenix;};

      generate = pkgs.writeShellScriptBin "generate" ''
        cd "$(${lib.getExe pkgs.git} rev-parse --show-toplevel)"

        ${lib.getExe pkgs.crate2nix} generate
      '';
    });

    nixosModules.default = import ./nix/module.nix self;

    defaultConfig = fromTOML (builtins.readFile ./config.toml);
    inherit (fromTOML (builtins.readFile ./users.toml)) users;

    devShells = eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      default = pkgs.mkShell {
        CONFIG_PATH = "config.dev.toml:users.toml";
        RUST_LOG = "warn,aocbot=trace";

        packages = [pkgs.crate2nix self.packages.${system}.generate];
      };
    });

    checks = builtins.mapAttrs (_: pkgs: pkgs // {_inputs = builtins.mapAttrs (_: drv: drv.inputDerivation) pkgs;}) self.packages;
  };
}
