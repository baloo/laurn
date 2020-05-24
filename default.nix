{ release ? true
, doCheck ? false
}:

let 
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs { };
  cargo_nix = pkgs.callPackage ./Cargo.nix { inherit pkgs release; };
  laurn_build = cargo_nix.rootCrate.build.override {
    runTests = doCheck;
  };
in laurn_build
