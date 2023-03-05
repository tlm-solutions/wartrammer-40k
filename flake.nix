{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    utils = {
      url = "github:numtide/flake-utils";
    };
  };

  outputs = { self, nixpkgs, nixpkgs-unstable, naersk, utils, ... }:
    utils.lib.eachDefaultSystem
      (system:
        let
          # flutterPackages has been updated to the version we need after 22.05
          # this is here so we do not overlay from nixpkgs-unstable after the update to 22.11
          pkgs = let
            pkgs = import nixpkgs { inherit system; };
            overlays = if pkgs.lib.hasPrefix "22.05" pkgs.lib.version
            then [ (self: super: { flutterPackages = (super.callPackage "${nixpkgs-unstable}/pkgs/development/compilers/flutter" { }); }) ]
            else [ ];
          in import nixpkgs { inherit system overlays; };

          backend = pkgs.callPackage ./pkgs/backend.nix {
            naersk = naersk.lib.${system};
          };
          frontend-fakeHash = pkgs.callPackage ./pkgs/frontend.nix { } { };
          frontend = pkgs.callPackage ./pkgs/frontend.nix { } { vendorHash = "sha256-PKzvqyPYNHnwmMBLHsSVjC6482x8yNesO+5oxLkwI9E="; };
        in
        rec {
          checks = packages;
          packages = {
            wartrammer-backend = backend;
            wartrammer-frontend-fakeHash = frontend-fakeHash;
            wartrammer-frontend = frontend;
            default = backend;
          };
          devShells = rec {
            default = backend;
            backend = pkgs.mkShell {
              nativeBuildInputs = (with packages.wartrammer-backend; nativeBuildInputs ++ buildInputs);
            };
          };
        }
      ) // {
      overlays.default = final: prev: {
        inherit (self.packages.${prev.system})
        wartrammer-backend;
        # wartrammer frontend needs to be build on x86_64-linux, but the output is generic html/js
        inherit (self.packages."x86_64-linux")
        wartrammer-frontend;
      };

      nixosModules = rec {
        default = wartrammer;
        wartrammer = import ./nixos-module;
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
