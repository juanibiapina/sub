{ nixpkgs ? <nixpkgs>
, pkgs ? import nixpkgs { }
}:

with pkgs;
rustPlatform.buildRustPackage rec {
  name = "sub-${version}";
  version = "1.0.0";
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
