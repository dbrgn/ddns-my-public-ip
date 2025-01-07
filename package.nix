{ rustPlatform, bind, ... }:
rustPlatform.buildRustPackage {
  pname = "ddns-my-public-ip";
  version = "0.1.0";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
  buildInputs = [ bind ]; # Dependency: nsupdate
  shellHook = ''
    export NSUPDATE="${bind}"
    export DNS_SERVER="${bind}"
  '';
}
