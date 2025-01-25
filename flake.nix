{
  description = "A CLI batch downloader for your Bandcamp collection.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    naersk.url = "github:nix-community/naersk/master";
  };

  outputs = {
    fenix,
    nixpkgs,
    naersk,
    ...
  }: let
    forSystems = fn:
      nixpkgs.lib.genAttrs [
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
        "x86_64-linux"
      ] (system: fn nixpkgs.legacyPackages.${system});
    defaultForSystems = fn: forSystems (pkgs: {default = fn pkgs;});

    mkBandsnatch = pkgs: let
      fenixPkgs = fenix.packages.${pkgs.system};
      toolchain = fenixPkgs.stable.defaultToolchain;
      naerskLib = pkgs.callPackage naersk {
        cargo = toolchain;
        rustc = toolchain;
      };
    in
      naerskLib.buildPackage {
        src = ./.;
      };
  in {
    packages = defaultForSystems (pkgs: mkBandsnatch pkgs);

    devShells = defaultForSystems (
      pkgs:
        pkgs.mkShell {
          inputsFrom = [(mkBandsnatch pkgs)];
          RUST_SRC_PATH = "${fenix.packages.${pkgs.system}.stable.rust-src}/lib/rustlib/src/rust/library";
        }
    );
  };
}
