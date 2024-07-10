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
    buildTargets = {
      "x86_64-linux" = "x86_64-unknown-linux-musl";
      "aarch64-linux" = "aarch64-unknown-linux-musl";
      "x86_64-darwin" = "x86_64-apple-darwin";
      "aarch64-darwin" = "aarch64-apple-darwin";
    };

    systems = builtins.attrNames buildTargets;

    # forSystems [...system] (system: ...)
    forSystems = systems: fn:
      nixpkgs.lib.genAttrs systems (system: fn system);

    # crossForSystems [...system] (hostSystem: targetSystem: ...)
    crossForSystems = systems: fn:
      forSystems systems (
        hostSystem:
          builtins.foldl'
          (acc: targetSystem:
            acc
            // {
              "cross-${targetSystem}" = fn hostSystem targetSystem;
            })
          {default = fn hostSystem hostSystem;}
          systems
      );

    mkBandsnatch = hostSystem: targetSystem: let
      rustTarget = buildTargets.${targetSystem};
      pkgs = import nixpkgs {system = hostSystem;};
      pkgsCross = import nixpkgs {
        system = hostSystem;
        crossSystem.config = rustTarget;
      };
      fenixPkgs = fenix.packages.${hostSystem};
      toolchain = fenixPkgs.combine [
        fenixPkgs.stable.rustc
        fenixPkgs.stable.cargo
        fenixPkgs.targets.${rustTarget}.stable.rust-std
      ];

      naersk-lib = pkgs.callPackage naersk {
        cargo = toolchain;
        rustc = toolchain;
      };
      TARGET_CC = "${pkgsCross.stdenv.cc}/bin/${pkgsCross.stdenv.cc.targetPrefix}cc";
    in
      naersk-lib.buildPackage {
        src = ./.;
        strictDeps = true;
        doCheck = false;

        inherit TARGET_CC;

        CARGO_BUILD_TARGET = rustTarget;
        CARGO_BUILD_RUSTFLAGS = [
          "-C"
          "target-feature=+crt-static"
          "-C"
          "linker=${TARGET_CC}"
        ];
      };
  in {
    packages = crossForSystems systems mkBandsnatch;

    devShells = forSystems systems (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        fenixPkgs = fenix.packages.${system};
      in {default = pkgs.mkShell {nativeBuildInputs = [fenixPkgs.stable.toolchain];};}
    );
  };
}
