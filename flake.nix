{
  description = "Update DNS records with your current public IP";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs =
    { self, nixpkgs }:
    let
      # Supported target systems
      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      # Helper to build a package for all supported systems above
      forAllSystems =
        f: nixpkgs.lib.genAttrs allSystems (system: f { pkgs = import nixpkgs { inherit system; }; });

      mkPackage = pkgs: pkgs.callPackage ./package.nix { };
    in
    {
      formatter = forAllSystems ({ pkgs }: pkgs.nixfmt-rfc-style);

      nixosModules.default = import ./nixos-module.nix self;

      overlays.default = final: _prev: { ddns-my-public-ip = mkPackage final; };

      packages = forAllSystems (
        { pkgs }:
        {
          default = mkPackage pkgs;
        }
      );
    };
}
