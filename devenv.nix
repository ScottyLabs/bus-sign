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

  git-hooks.hooks = {
    deno-check.entry = lib.mkForce "bash -c 'cd frontend && deno check .'";
    deno-test.entry = lib.mkForce "deno test --ignore=.devenv,.direnv --permit-no-files";
  };
}
