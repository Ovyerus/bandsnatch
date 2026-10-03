{
  lib,
  rustPlatform,
  stdenv,
  pkgsStatic,
}: let
  manifest = lib.importTOML ./Cargo.toml;
in
  rustPlatform.buildRustPackage {
    pname = manifest.package.name;
    version = manifest.package.version;

    src = lib.fileset.toSource {
      root = ./.;
      fileset = lib.fileset.unions [./Cargo.toml ./Cargo.lock ./src];
    };
    cargoLock.lockFile = ./Cargo.lock;

    # Keep Darwin releases standalone while retaining Apple's system runtime.
    buildInputs = lib.optionals stdenv.hostPlatform.isDarwin [pkgsStatic.libiconv];

    strictDeps = true;

    meta = {
      inherit (manifest.package) description homepage;
      license = lib.licenses.mit;
      mainProgram = "bandsnatch";
      platforms = ["aarch64-darwin" "aarch64-linux" "x86_64-linux"];
    };
  }
