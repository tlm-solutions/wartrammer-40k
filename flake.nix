{
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;

    naersk = {
      url = github:nix-community/naersk;
      inputs.nixpkgs.follows = "nixpkgs";
    };

    utils = {
      url = github:numtide/flake-utils;
      inputs.nixpkgs.follows = "nixpkgs";
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
          frontend = pkgs.callPackage ./pkgs/frontend.nix { };

        in
        rec {
          checks = packages;
          packages.wartrammer-backend = backend;
          packages.wartrammer-frontend = frontend;
          defaultPackage = backend;
          overlay = (final: prev: {
            wartrammer-backend = backend;
            wartrammer-frontend = frontend;
          });
        }
      ) // {
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
