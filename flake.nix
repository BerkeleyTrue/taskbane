{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    nixgl.url = "github:nix-community/nixGL";
    flake-parts.url = "github:hercules-ci/flake-parts";
    boulder.url = "github:berkeleytrue/nix-boulder-banner";
  };

  outputs = inputs @ {
    flake-parts,
    nixpkgs,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.boulder.flakeModule
      ];
      systems = ["x86_64-linux"];
      perSystem = {
        config,
        system,
        ...
      }: let
        pkgs = import nixpkgs {
          inherit system;

          overlays = [
          ];
        };
      in {
        formatter.default = pkgs.alejandra;
        boulder.commands = [
          {
            exec = pkgs.writeShellScriptBin "run" ''
              cargo run
            '';
            description = "cargo run";
          }
          {
            exec = pkgs.writeShellScriptBin "build" ''
              cargo build
            '';
            description = "cargo build";
          }
          {
            exec = pkgs.writeShellScriptBin "watch" ''
              cargo watch -x run
            '';
            description = "cargo watch -x run";
          }
        ];
        devShells.default = pkgs.mkShell {
          name = "rust";
          inputsFrom = [
            config.boulder.devShell
          ];

          buildInputs = with pkgs; [
            cargo
            cargo-generate
            rustc
            pnpm
            tailwindcss_4
            # rustup
            # rustfmt
          ];
        };
      };
      flake = {};
    };
}
