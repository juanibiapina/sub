{ pkgs }:

with pkgs;
rustPlatform.buildRustPackage rec {
  name = "sub-${version}";
  version = "2.3.0";
  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  meta = with lib; {
    description = "Organize groups of scripts into documented CLIs with subcommands";
    homepage = "https://github.com/juanibiapina/sub";
    license = licenses.mit;
  };

}
