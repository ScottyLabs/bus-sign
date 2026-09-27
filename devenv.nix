{ lib, inputs, ... }:

{
  imports = [ inputs.scottylabs.devenvModules.default ];

  scottylabs = {
    enable = true;
    project.name = "bus-sign";
    rust.enable = true;
    deno.enable = true;
    secrets.enable = true;
    kennel.services.backend = {
      customDomain = "bus-sign.scottylabs.org";
    };
  };

  env.STATIC_DIR = "frontend/dist";

  processes = {
    backend.exec = "cargo run -p backend";
    frontend.exec = ''
      cd frontend
      deno install
      VITE_API_BASE=http://localhost:''${PORT:-8080} deno task dev
    '';
  };

  git-hooks.hooks = {
    deno-check.entry = lib.mkForce "bash -c 'cd frontend && deno check .'";
    deno-test.entry = lib.mkForce "deno test --ignore=.devenv,.direnv --permit-no-files";
  };
}
