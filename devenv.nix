{ pkgs, lib, config, inputs, ... }:

{
  packages = [
      pkgs.git
      pkgs.libffi
      pkgs.opam
  ];

  languages.ocaml = {
      enable = true;
      lsp.enable = true;
  };
}
