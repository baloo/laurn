let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs { };
  lib = pkgs.lib;
  deps = import ./default.nix;
  crate2nix = pkgs.callPackage sources.crate2nix { };
in
pkgs.mkShell {
  name = "laurn-shell";
  nativeBuildInputs = [
    crate2nix
  ];
  buildInputs = [
    crate2nix
  ];
}
