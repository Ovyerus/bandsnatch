{
  description = "A CLI batch downloader for your Bandcamp collection.";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = {nixpkgs, ...}: let
    systems = ["aarch64-darwin" "aarch64-linux" "x86_64-linux"];
    forSystems = nixpkgs.lib.genAttrs systems;
  in {
    packages = forSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      packagePkgs = if pkgs.stdenv.hostPlatform.isLinux then pkgs.pkgsStatic else pkgs;
    in {
      default = packagePkgs.callPackage ./package.nix {};
    });

    devShells = forSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      default = pkgs.mkShell {
        packages = [pkgs.cargo pkgs.rustc pkgs.rustfmt pkgs.clippy];
        buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
          pkgs.pkgsStatic.libiconv
        ];
      };
    });
  };
}
