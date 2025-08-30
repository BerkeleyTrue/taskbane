{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    nixgl.url = "github:nix-community/nixGL";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {
    flake-parts,
    nixpkgs,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [];
      systems = ["x86_64-linux"];
      perSystem = {system, ...}: let
        pkgs = import nixpkgs {
          inherit system;

          overlays = [
          ];
        };
      in {
        formatter.default = pkgs.alejandra;
        devShells.default = pkgs.mkShell {
          name = "taskbane";

          buildInputs = with pkgs; [
            cargo
            cargo-generate
            rustc
            sqlx-cli
            pnpm
            tailwindcss_4
            openssl
            pkg-config
            # rustup
            # rustfmt
          ];

          shellHook = ''
            function menu () {
              echo
              echo -e "\033[1;34m>==> ️  '$name' shell\n\033[0m"
              just --list
              echo
              echo "(Run 'just --list' to display this menu again)"
              echo
            }

            menu
          '';
        };
      };
      flake = {};
    };
}
