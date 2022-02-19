# import niv sources and the pinned nixpkgs
{ sources ? import ./nix/sources.nix, pkgs ? import sources.nixpkgs { } }:
let
  # import rust compiler
  rust = import ./nix/rust.nix { inherit sources; };

  # configure naersk to use our pinned rust compiler
  naersk = pkgs.callPackage sources.naersk {
    rustc = rust;
    cargo = rust;
  };

  # tell nix-build to ignore the `target` directory
  src = builtins.filterSource
    (path: type: type != "directory" || builtins.baseNameOf path != "target")
    ./.;
in
naersk.buildPackage {
  inherit src;

  buildInputs = [
    pkgs.openssl.dev
    pkgs.darwin.apple_sdk.frameworks.Security
    pkgs.darwin.apple_sdk.frameworks.CoreServices
    pkgs.sqlite
    rust
  ];

  OPENSSL_DIR = "${pkgs.openssl.dev}";
  OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";

  remapPathPrefix =
    true; # remove nix store references for a smaller output package
}