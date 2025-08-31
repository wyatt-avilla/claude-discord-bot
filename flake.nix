{
  description = "Talk to Claude in Discord";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";

    nix-checks = {
      url = "github:wyatt-avilla/nix-ci";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, ... }@inputs:
    let
      nixosModules = {
        claude-discord-bot = import ./service.nix { inherit self; };
      };
    in
    {
      inherit nixosModules;
    }
    // inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import inputs.rust-overlay) ];
        };

        nativeRustToolchain = with pkgs; [
          (rust-bin.stable.latest.default.override {
            extensions = [
              "clippy"
              "rust-src"
            ];
          })
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs =
            nativeRustToolchain
            ++ (with pkgs; [
              rust-analyzer
              openssl
              pkg-config
            ]);
          buildInputs = [ ];
        };

        packages.default =
          let
            cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          in
          pkgs.rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            inherit (cargoToml.package) version;

            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            checkPhase = ''
              cargo clippy -- -W clippy::pedantic -D warnings
              cargo fmt --check
              cargo test
            '';

            nativeBuildInputs =
              nativeRustToolchain
              ++ (with pkgs; [
                openssl
                pkg-config
              ]);

            buildInputs = with pkgs; [
              openssl
              pkg-config
            ];
          };

        checks = {
          formatting = inputs.nix-checks.lib.mkFormattingCheck {
            inherit pkgs;
            src = self;
          };

          linting = inputs.nix-checks.lib.mkLintingCheck {
            inherit pkgs;
            src = self;
          };

          dead-code = inputs.nix-checks.lib.mkDeadCodeCheck {
            inherit pkgs;
            src = self;
          };
        };
      }
    );
}
