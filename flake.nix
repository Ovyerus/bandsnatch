{
  description = "A CLI batch downloader for your Bandcamp collection.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    crane,
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
      rustBin = rust-overlay.lib.mkRustBin {} pkgs;
      craneLib = (crane.mkLib pkgs).overrideToolchain (p: rustBin.stable.latest.default);
    in
      pkgs.callPackage ./package.nix {inherit craneLib;};
  in {
    packages = defaultForSystems mkBandsnatch;

    devShells = defaultForSystems (
      pkgs: pkgs.mkShell {inputsFrom = [(mkBandsnatch pkgs)];}
    );
  };
}
