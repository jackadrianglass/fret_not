{ pkgs, lib, config, inputs, ... }:

{
  packages = [
      pkgs.git
  ];

  languages.rust = {
    enable = true;
    lsp.enable = true;
  };

  git-hooks.hooks = {
    rustfmt.enable = true;
  };
}
