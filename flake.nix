{
  description = "qc";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-22.05";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        inherit (pkgs) lib;

        craneLib = crane.lib.${system};
        src = ./.;

        commonBuildInputs = [
          pkgs.libiconv
          pkgs.postgresql
          pkgs.openssl.dev
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.CoreServices
          pkgs.darwin.apple_sdk.frameworks.CoreFoundation
        ];


        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src;
          buildInputs = commonBuildInputs;
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        };

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        qc = craneLib.buildPackage {
          inherit cargoArtifacts src;
          buildInputs = commonBuildInputs;

          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          # doCheck = false;
        };
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit qc;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          qc-clippy = craneLib.cargoClippy {
            inherit cargoArtifacts src;
            cargoClippyExtraArgs = "-- --deny warnings";
          };

          # Check formatting
          qc-fmt = craneLib.cargoFmt {
            inherit src;
          };
        } // lib.optionalAttrs (system == "x86_64-linux") {
          # NB: cargo-tarpaulin only supports x86_64 systems
          # Check code coverage (note: this will not upload coverage anywhere)
          qc-coverage = craneLib.cargoTarpaulin {
            inherit cargoArtifacts src;
          };
        };

        packages.default = qc;

        apps.default = flake-utils.lib.mkApp {
          drv = qc;
        };

        packages.container = pkgs.dockerTools.buildLayeredImage {
          name = "qc";
          tag = qc.version;
          created = "now";
          contents = qc;
          config.Cmd = [ "${qc}/bin/qc" ];
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          buildInputs = commonBuildInputs;
          nativeBuildInputs = with pkgs; [
            cargo
            rustc
          ];
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";

        };
      });
}
