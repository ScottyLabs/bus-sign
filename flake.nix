{
  description = "CUC Bus Sign";

  nixConfig = {
    extra-substituters = [ "https://scottylabs.cachix.org" ];
    extra-trusted-public-keys = [
      "scottylabs.cachix.org-1:hajjEX5SLi/Y7yYloiXTt2IOr3towcTGRhMh1vu6Tjg="
    ];
  };

  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    devenv.url = "github:cachix/devenv";
    bun2nix = {
      url = "github:nix-community/bun2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, devenv, bun2nix, ... }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = pkgsFor system;
        in
        {
          devenv = devenv.packages.${system}.devenv;
        }
        // (nixpkgs.lib.optionalAttrs (system == "x86_64-linux") (
          let
            b2n = bun2nix.packages.${system}.default;

            busSignFrontend = b2n.mkDerivation {
              pname = "bus-sign-frontend";
              version = (builtins.fromJSON (builtins.readFile ./frontend/package.json)).version;
              src = ./frontend;

              bunDeps = b2n.fetchBunDeps {
                bunNix = ./frontend/bun.nix;
              };

              buildPhase = ''
                bun run build
              '';

              installPhase = ''
                mkdir -p $out
                cp -r dist/* $out/
              '';
            };

            cargoNix = pkgs.callPackage ./Cargo.nix { };

            busSignBackend = cargoNix.rootCrate.build;

          in
          {
            inherit busSignFrontend busSignBackend;
            default = busSignBackend;
          }
        ))
      );
    };
}
