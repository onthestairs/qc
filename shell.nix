let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  pkgs = import sources.nixpkgs { };
in
pkgs.mkShell {
  buildInputs = [
    pkgs.openssl.dev
    pkgs.darwin.apple_sdk.frameworks.Security
    pkgs.darwin.apple_sdk.frameworks.CoreServices
    pkgs.darwin.apple_sdk.frameworks.CoreFoundation
    pkgs.sqlite
    pkgs.libiconv
    rust
  ];

  OPENSSL_DIR = "${pkgs.openssl.dev}";
  OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";

  shellHook = ''
    export NIX_LDFLAGS="-F${pkgs.darwin.apple_sdk.frameworks.CoreFoundation}/Library/Frameworks -framework CoreFoundation $NIX_LDFLAGS";
  '';

  # env vars for the server
  RUST_BACKTRACE = "1";
}
