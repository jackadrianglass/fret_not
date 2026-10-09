{ pkgs, lib, config, inputs, ... }:

{
  packages = [
      pkgs.git
      pkgs.libffi
      pkgs.opam
      pkgs.SDL2
      pkgs.SDL2_ttf
      pkgs.SDL2_image
      pkgs.pkg-config
  ];

  languages.ocaml = {
      enable = true;
      lsp.enable = true;
  };
}
