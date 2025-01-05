{
  description = "Update DNS records with your current public IP";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }:
    let
      # Supported target systems
      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      # Helper to build a package for all supported systems above
      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
        pkgs = import nixpkgs { inherit system; };
      });
    in
    {
      # options = {
      #   dnsServer = lib.mkOption {
      #     type = lib.types.str;
      #     description = "DNS server to update";
      #     example = "example.com";
      #   };
      #   dnsZone = lib.mkOption {
      #     type = lib.types.str;
      #     description = "DNS zone to update (no trailing dot)";
      #     example = "site1.example.com";
      #   };
      #   domains = lib.mkOption {
      #     type = lib.types.str;
      #     description = "Comma-separated list of domains to update (no trailing dot)";
      #     example = "ip.site1.example.com,webcam.site1.example.com";
      #   };
      # };

      packages = forAllSystems ({ pkgs }: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "ddns-my-public-ip";
          version = "0.1.0";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = [ pkgs.bind ]; # Dependency: nsupdate
          shellHook = ''
            export NSUPDATE="${pkgs.bind}"
            export DNS_SERVER="${pkgs.bind}"
          '';
        };
      });
    };
}
