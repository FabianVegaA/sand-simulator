{ pkgs, lib, config, inputs, fenix, ... }:

{
  packages = [
    pkgs.git
    pkgs.trunk
    pkgs.openssl
  ];

  languages.rust = {
      enable = true;
      channel = "nightly";
      rustflags = "-Z threads=8";
      targets = [ "wasm32-unknown-unknown" ];
    };

  processes.server.exec = "${pkgs.trunk}/bin/trunk serve";
}
