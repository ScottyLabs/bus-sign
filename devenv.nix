{ pkgs, config, inputs, ... }:

{
  cachix.pull = [ "scottylabs" ];

  packages = [
    inputs.bun2nix.packages.${pkgs.stdenv.system}.default
  ] ++ (with pkgs; [
    bun
    crate2nix
  ]);

  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
      "rust-src"
    ];
  };

  dotenv.enable = true;

  treefmt = {
    enable = true;
    config.programs = {
      nixpkgs-fmt = {
        enable = true;
        excludes = [ "Cargo.nix" "bun.nix" ];
      };
      rustfmt.enable = true;
    };
    config.settings.formatter.biome = {
      command = "${pkgs.biome}/bin/biome";
      options = [ "check" "--write" "--no-errors-on-unmatched" "--config-path" "${config.devenv.root}/biome.json" ];
      includes = [ "*.js" "*.ts" "*.mjs" "*.mts" "*.cjs" "*.cts" "*.jsx" "*.tsx" "*.d.ts" "*.d.cts" "*.d.mts" "*.json" "*.jsonc" "*.css" ];
    };
  };

  git-hooks.hooks = {
    treefmt.enable = true;
    clippy = {
      enable = true;
      packageOverrides.cargo = config.languages.rust.toolchainPackage;
      packageOverrides.clippy = config.languages.rust.toolchainPackage;
    };
    cargo-nix-update = {
      enable = true;
      name = "cargo-nix-update";
      entry = "${pkgs.writeShellScript "cargo-nix-update" ''
        if git diff --cached --name-only | grep -q 'Cargo\.\(toml\|lock\)'; then
          ${pkgs.crate2nix}/bin/crate2nix generate
          git add Cargo.nix
        fi
      ''}";
      files = "Cargo\\.(toml|lock)$";
      language = "system";
      pass_filenames = false;
    };
  };
}
