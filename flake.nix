{
  description = "CUC Bus Sign";

  nixConfig = {
    extra-substituters = [ "https://scottylabs.cachix.org" ];
    extra-trusted-public-keys = [
      "scottylabs.cachix.org-1:hajjEX5SLi/Y7yYloiXTt2IOr3towcTGRhMh1vu6Tjg="
    ];
  };

  inputs = {
    scottylabs.url = "git+https://git.cmu.dev/ScottyLabs/kennel";
    nixpkgs.follows = "scottylabs/nixpkgs";
    flake-parts.follows = "scottylabs/flake-parts";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.scottylabs.flakeModules.default ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      perSystem =
        { helpers, ... }:
        let
          frontend = helpers.buildDenoTask {
            src = ./frontend;
            pname = "bus-sign-frontend";
            output = "build";
          };
        in
        {
          scottylabs = {
            project.name = "bus-sign";
            rust.enable = true;
            deno = {
              enable = true;
              svelte.enable = true;
              dirs = [ "frontend" ];
            };

            processes = {
              backend.run = "secretspec run -- cargo run -p backend";
              frontend.run = "cd frontend && deno task dev";
            };
          };

          services.backend = {
            package = helpers.buildRustService {
              src = ./.;
              pname = "backend";
              buildArgs.cargoExtraArgs = "--locked -p backend";
            };
            customDomain = "bus-sign.scottylabs.org";
            env.STATIC_DIR = "${frontend}";
          };
        };
    };
}
