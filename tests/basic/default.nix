{ system ? builtins.currentSystem }:

let
  pkgs = import <nixpkgs> { inherit system; };
  bash = pkgs.bash;
  coreutils = pkgs.coreutils;
  procps = pkgs.procps;
  iproute = pkgs.iproute;
in pkgs.stdenv.mkDerivation rec {
    name = "basic-test";
    buildInputs = [
        coreutils
	procps
        bash
        iproute
    ];

    src = ./src;
    binpath = pkgs.lib.makeBinPath [
        coreutils procps iproute
    ];

    buildPhase = "";
    installPhase = ''
      cp -r ./start.sh $out
      chmod +x $out
      substituteAllInPlace $out
    '';
}
