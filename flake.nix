{
  description = "CUC Bus Sign";

  nixConfig = {
    extra-substituters = [ "https://scottylabs.cachix.org" ];
    extra-trusted-public-keys = [
      "scottylabs.cachix.org-1:hajjEX5SLi/Y7yYloiXTt2IOr3towcTGRhMh1vu6Tjg="
    ];
  };

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    scottylabs = {
      url = "git+https://codeberg.org/ScottyLabs/kennel";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      scottylabs,
      ...
    }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
      ];
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          helpers = scottylabs.mkLib pkgs;

          frontend = helpers.buildDenoTask {
            src = ./frontend;
            pname = "frontend";
            task = "build";
          };

          backend = helpers.buildRustService {
            src = ./.;
            pname = "backend";
            nativeBuildInputs = [ pkgs.makeWrapper ];
            buildArgs = {
              cargoExtraArgs = "-p backend";
              postInstall = ''
                mkdir -p $out/share/bus-sign/www
                cp -r ${frontend}/* $out/share/bus-sign/www/
                chmod -R u+w $out/share/bus-sign/www
                wrapProgram $out/bin/backend \
                  --set-default STATIC_DIR $out/share/bus-sign/www
              '';
            };
          };
        in
        {
          inherit frontend backend;
        }
      );
    };
}
