{
  description = "A bullet heaven game";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, fenix, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        toolchain = with fenix.packages.${system}; fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-hdc9s5/xycLxDZc44maAzGdxD4ZZuo2/he0DagUmM+c=";
        };
      in {
        devShell = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            # build tooling
            toolchain
            clang_14
            mold
            cargo-nextest
            just
            trunk
            binaryen
            graphviz # useful for bevy_mod_debugdump

            # libraries necessary for bevy
            pkg-config
            libxkbcommon
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            alsa-lib
            udev
            vulkan-loader
            wayland
            openssl

            cargo-bloat
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeBuildInputs;
        };
      }
    );
}
