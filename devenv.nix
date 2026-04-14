{ pkgs, config, inputs, ... }:

let
  cargoNix = pkgs.callPackage ./Cargo.nix { };
  busSignBackend = cargoNix.rootCrate.build;
in
{
  imports = [ inputs.scottylabs.devenvModules.default ];

  scottylabs = {
    enable = true;
    project.name = "bus-sign";
    rust.enable = true;
    bun.enable = true;
  };

  packages = [
    inputs.bun2nix.packages.${pkgs.stdenv.system}.default
  ];

  outputs = { inherit busSignBackend; };
}
