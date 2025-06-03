{
  description = "Rust + Wayland dev environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Wayland/GL runtime libraries
        libPath = with pkgs; lib.makeLibraryPath [
          libGL
          libxkbcommon
          wayland
        ];
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rust-bin.stable.latest.default
            pkgs.rust-analyzer
            pkgs.pkg-config  # recommended for wayland-client builds
          ];

          # Environment variables needed for runtime stuff
          LD_LIBRARY_PATH = libPath;
          RUST_LOG = "debug";
        };
      });
}
