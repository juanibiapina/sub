{
  description = "Organize groups of scripts into documented CLIs with subcommands";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      rec {
        packages = {
          default = packages.sub;
          sub = pkgs.callPackage ./default.nix { inherit pkgs; };
        };

        apps = {
          default = apps.sub;
          sub = flake-utils.lib.mkApp {
            drv = packages.sub;
          };
        };

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc cargo rust-analyzer rustfmt
          ];
        };
      }
    );
}
