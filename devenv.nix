{ pkgs, lib, config, inputs, ... }:

{
  packages = [
      pkgs.git
  ];

  languages.rust = {
    enable = true;
    lsp.enable = true;
  };
}
