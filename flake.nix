{
  description = "A CLI batch downloader for your Bandcamp collection.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        # TODO: musl https://github.com/ipetkov/crane/blob/master/examples/cross-musl/flake.nix
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        # TODO: cross compilation
        craneLib = crane.lib.${system};
        # TODO: lock
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src"];
        };

        stdenv =
          if pkgs.stdenv.isLinux
          then pkgs.stdenv
          else pkgs.clangStdenv;

        commonArgs = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);

          buildInputs = with pkgs;
            []
            ++ (lib.optional stdenv.isDarwin [libiconvReal]);

          nativeBuildInputs = with pkgs;
            [pkg-config]
            ++ (lib.optional stdenv.isLinux [openssl.dev])
            ++ (lib.optional stdenv.isDarwin (with darwin.apple_sdk; [
              frameworks.AppKit
              frameworks.CoreFoundation
            ]));
        };

        artifacts = craneLib.buildDepsOnly commonArgs;

        bandsnatch = craneLib.buildPackage (commonArgs // {inherit artifacts;});
      in {
        packages.default = bandsnatch;
        devShells.default = with pkgs;
          mkShell {
            nativeBuildInputs = commonArgs.nativeBuildInputs;
            buildInputs = [rust] ++ commonArgs.buildInputs;
          };
      }
    );
}
