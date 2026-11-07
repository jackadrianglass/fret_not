{ pkgs, lib, config, inputs, ... }:

{
  packages = [
      pkgs.git
      pkgs.just
      pkgs.nushell
  ];

  languages.rust = {
    enable = true;
    lsp.enable = true;
  };

  git-hooks.hooks = {
    rustfmt.enable = true;
  };
}
