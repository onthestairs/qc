{ system ? builtins.currentSystem }:

let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs { };
  qc = import ./qc.nix { inherit sources pkgs; };

  name = "onthestairs/qc";
  tag = "latest";

in
pkgs.dockerTools.buildLayeredImage {
  inherit name tag;
  contents = [ qc ];

  config = {
    Cmd = [ "/bin/server" ];
    Env = [ ];
    WorkingDir = "/";
  };
}