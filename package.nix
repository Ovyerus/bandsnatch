{
  craneLib,
  darwin,
  lib,
  libiconvReal,
  stdenv,
}: let
  src = craneLib.cleanCargoSource (craneLib.path ./.);
  buildInputs = lib.optionals stdenv.isDarwin [libiconvReal];
  nativeBuildInputs =
    []
    ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [AppKit CoreFoundation]);
in
  craneLib.buildPackage {
    inherit src buildInputs nativeBuildInputs;
    # strictDeps = true;
  }
