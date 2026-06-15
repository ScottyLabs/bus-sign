{ pkgs, config, inputs, ... }:

let
  b2n = inputs.bun2nix.packages.${pkgs.stdenv.system}.default;

  frontend = b2n.mkDerivation {
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
  backend = cargoNix.rootCrate.build.overrideAttrs (old: {
    postInstall = (old.postInstall or "") + ''
      cp -r ${frontend} $out/static
    '';
  });
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

  cachix.enable = false;

  packages = [
    inputs.bun2nix.packages.${pkgs.stdenv.system}.default
  ];

  outputs = { inherit backend; };

  processes.backend.exec = "secretspec run --profile dev -- cargo run -p scottylabs_bus_backend";

  env.VAULT_ADDR = "https://secrets2.scottylabs.org";
}
