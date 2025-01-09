{ rustPlatform, bind, ... }:
rustPlatform.buildRustPackage {
  pname = "ddns-my-public-ip";
  version = "0.1.0";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
  buildInputs = [ bind.dnsutils ]; # Dependency: nsupdate
  preBuild = ''
    export NSUPDATE="${bind.dnsutils}/bin/nsupdate"
  '';
}
