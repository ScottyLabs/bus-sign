{ pkgs, config, inputs, ... }:

let
  cargoNix = pkgs.callPackage ./Cargo.nix { };
  backend = cargoNix.rootCrate.build;
in
{
  imports = [ inputs.scottylabs.devenvModules.default ];

  scottylabs = {
    enable = true;
    project.name = "bus-sign";
    rust.enable = true;
    bun.enable = true;
    secrets.enable = true;
    kennel.services.backend = {
      customDomain = "bus-sign.scottylabs.org";
    };
  };

  packages = [
    inputs.bun2nix.packages.${pkgs.stdenv.system}.default
  ];

  outputs = { inherit backend; };
}
