{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    nixgl.url = "github:nix-community/nixGL";
    flake-parts.url = "github:hercules-ci/flake-parts";

    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs @ {
    flake-parts,
    nixpkgs,
    git-hooks,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.git-hooks.flakeModule
      ];
      systems = ["x86_64-linux"];
      perSystem = {
        system,
        config,
        ...
      }: let
        pkgs = import nixpkgs {
          inherit system;

          overlays = [
          ];
        };

        taskbane = pkgs.rustPlatform.buildRustPackage {
          pname = "taskbane";
          version = "0.1.0";
          src = builtins.path {
            path = ./.;
            name = "taskbane-src";
            filter = path: type:
              !builtins.elem (builtins.baseNameOf path) [
                "target"
                "data"
                "node_modules"
              ];
          };

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [pkg-config];
          buildInputs = with pkgs; [openssl sqlite];

          SQLX_OFFLINE = "true";

          postInstall = ''
            mkdir -p $out/share/taskbane
            cp -r public $out/share/taskbane/public
          '';
        };
      in {
        packages.default = taskbane;

        formatter.default = pkgs.alejandra;

        pre-commit.settings.hooks.alejandra.enable = true;
        pre-commit.settings.hooks.clippy.enable = true;
        pre-commit.settings.hooks.rustfmt.enable = true;

        devShells.default = pkgs.mkShell {
          name = "taskbane";

          buildInputs = with pkgs; [
            cargo
            cargo-generate
            cargo-watch
            rustc

            clippy
            rustfmt

            sqlx-cli
            nodejs
            pnpm
            tailwindcss_4
            openssl
            pkg-config
            sqlite
            taskchampion-sync-server
          ];

          shellHook = ''
            ${config.pre-commit.installationScript}

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
