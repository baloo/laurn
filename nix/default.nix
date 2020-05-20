{ system ? builtins.currentSystem }:

let
  pkgs = import <nixpkgs> { inherit system; };
  buildEnv = pkgs.mkShell;
  stdenv = pkgs.stdenv;
  lib = pkgs.lib;
in rec {
  dev = pkgs.mkShell {
    name = "laurn-shell";
    #paths = [ pkgs.cargo pkgs.rustc ];
    buildInputs = [
      pkgs.cargo
      pkgs.rustc
      pkgs.strace
      pkgs.glibc.bin
      pkgs.gcc
    ];
  };
}
