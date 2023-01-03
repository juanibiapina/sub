{
  description = "Organize groups of scripts into documented CLIs with subcommands";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system}; in
      rec {

        packages = {
          default = packages.sub;
          sub = pkgs.callPackage ./default.nix { inherit pkgs; };
        };

        lib = {
          mkSubDerivation = args@{ pname, cmd ? pname, buildInputs ? [ ], ... }:
            pkgs.stdenv.mkDerivation (args // rec {
              buildInputs = [ packages.sub pkgs.rsync ];

              buildPhase = "true";

              installPhase = ''
                mkdir -p $out/bin
                echo foo
                cat <<EOF >$out/bin/${cmd}
                #!/usr/bin/env bash
                set -e
                ${packages.sub}/bin/sub --name ${cmd} --absolute "$out/opt/${pname}" -- "\$@"
                EOF
                chmod a+x $out/bin/${cmd}

                mkdir -p $out/opt/${pname}/lib
                if [ -e lib ]; then
                  rsync -rp lib/ $out/opt/${pname}/lib/
                fi

                mkdir -p $out/opt/${pname}/libexec
                if [ -e libexec ]; then
                  rsync -rp libexec/ $out/opt/${pname}/libexec/
                fi

                cat <<EOF >$out/opt/${pname}/completions.zsh
                if [[ ! -o interactive ]]; then
                  return
                fi

                compctl -K _${cmd} ${cmd}

                _${cmd}() {
                  local words completions
                  read -cA words

                  if [ "\''${#words}" -eq 2 ]; then
                    completions="\$(${cmd} completions)"
                  else
                    completions="\$(${cmd} completions "\''${words[@]:1:-1}")"
                  fi

                  reply=("\''${(ps:\n:)completions}")
                }
                EOF
                chmod a+x $out/opt/${pname}/completions.zsh
              '';
            });
        };

        apps = {
          default = apps.sub;
          sub = flake-utils.lib.mkApp { drv = packages.sub; };
        };

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [ rustc cargo rust-analyzer rustfmt bats ];
        };
      });
}
