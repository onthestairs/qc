{ sources ? import ./sources.nix }:

let
  pkgs =
    import sources.nixpkgs { overlays = [ (import sources.nixpkgs-mozilla) ]; };
  channel = "nightly";
  date = "2022-02-14";
  targets = [ ];
  chan = pkgs.rustChannelOfTargets channel date targets;
in
chan.override {
  extensions = [
    "rust-src"
    "rls-preview"
    "rust-analysis"
    "clippy-preview"
    "rustfmt-preview"
  ];
}
