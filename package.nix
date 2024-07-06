{
  craneLib,
  darwin,
  lib,
  libiconvReal,
  openssl,
  pkg-config,
  stdenv,
}: let
  src = craneLib.cleanCargoSource (craneLib.path ./.);
  buildInputs = lib.optionals stdenv.isDarwin [libiconvReal];
  nativeBuildInputs =
    [pkg-config openssl.dev]
    ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [AppKit CoreFoundation]);
in
  craneLib.buildPackage {
    inherit src buildInputs nativeBuildInputs;
    # strictDeps = true;
  }
