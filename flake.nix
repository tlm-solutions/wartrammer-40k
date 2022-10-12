{
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;

    naersk = {
      url = github:nix-community/naersk;
      inputs.nixpkgs.follows = "nixpkgs";
    };

    utils = {
      url = github:numtide/flake-utils;
    };
  };

  outputs = { self, nixpkgs, naersk, utils, ... }:
    utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          backend = pkgs.callPackage ./pkgs/backend.nix {
            naersk = naersk.lib.${system};
          };
          frontend-fakeHash = pkgs.callPackage ./pkgs/frontend.nix { } { };
          frontend = pkgs.callPackage ./pkgs/frontend.nix { } { vendorHash = "sha256-9O5WbcOTlkbsRVsMIrhO5bnHc1Bp5ij+5HvP56oaw2s="; };
        in
        rec {
          checks = packages;
          packages = {
            wartrammer-backend = backend;
            wartrammer-frontend-fakeHash = frontend-fakeHash;
            wartrammer-frontend = frontend;
            default = backend;
          };
        }
      ) // {
      overlays.default = final: prev: {
        inherit (self.packages.${prev.system})
          wartrammer-backend
          wartrammer-frontend;
      };
      hydraJobs =
        let
          hydraSystems = [
            "x86_64-linux"
            "aarch64-linux"
          ];
        in
        builtins.foldl'
          (hydraJobs: system:
            builtins.foldl'
              (hydraJobs: pkgName:
                nixpkgs.lib.recursiveUpdate hydraJobs {
                  ${pkgName}.${system} = self.packages.${system}.${pkgName};
                }
              )
              hydraJobs
              (builtins.attrNames self.packages.${system})
          )
          { }
          hydraSystems;
    };
}
