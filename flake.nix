{

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    systems.url = "github:nix-systems/default";
    flake-utils = {
          url = "github:numtide/flake-utils";
          inputs.systems.follows = "systems";
    };
    fenix = {
          url = "github:nix-community/fenix";
          inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      fenix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          system = system;
          overlays = [ fenix.overlays.default ];
        };
        lib = pkgs.lib;
        rust =
          with fenix.packages.${system};
          fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-4ublOInImJyXpU1kMUSsAOUxT0fCuYgcs7M1i6N4L3k="; # Dunno how to update it automatically
          };
      in
      {
        formatter = pkgs.nixfmt-rfc-style;

        devShells.default = pkgs.mkShell {
          name = "rust-dev";
          packages = [
            rust
            pkgs.bashInteractive
            pkgs.nixfmt-rfc-style
            pkgs.pkg-config
            pkgs.protobuf
            pkgs.iconv
            pkgs.cargo-udeps
            pkgs.cmake
            pkgs.hwloc
          ] ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ];
        };
      }
    );
}
