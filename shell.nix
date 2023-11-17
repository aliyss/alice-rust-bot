{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = with pkgs; [ openssl pkgconfig cargo rustc rustfmt gcc clippy ];
  RUST_BACKTRACE = 1;
}
