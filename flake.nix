{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      imports = [
        inputs.treefmt-nix.flakeModule
      ];
      perSystem = { config, self', pkgs, lib, system, ... }:
        let
          buildInputs = with pkgs; [
            udev
            alsa-lib
            vulkan-loader
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            libxkbcommon
            wayland
          ];
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            clippy
            pkg-config
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

          naersk = pkgs.callPackage inputs.naersk { };

          xwin-toolchan = with inputs.fenix.packages.${system}; combine [
            minimal.cargo
            minimal.rustc
            targets."x86_64-pc-windows-msvc".latest.rust-std
          ];
          xwin-buildInputs = with pkgs; [
            xwin-toolchan
            clippy
            pkg-config
            cargo-xwin
          ];
        in
        {
          _module.args.pkgs = import inputs.nixpkgs { inherit system; };

          treefmt.config = {
            projectRootFile = "flake.nix";
            programs = {
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
              taplo.enable = true;
            };
          };

          # packages.default = naersk.buildPackage {
          #   src = ./.;
          #   inherit buildInputs nativeBuildInputs LD_LIBRARY_PATH;
          # };

          devShells.default = pkgs.mkShell {
            inherit buildInputs LD_LIBRARY_PATH;
            nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
              rustfmt
            ]);
          };

          devShells.xwin = pkgs.mkShell {
            buildInputs = xwin-buildInputs;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath xwin-buildInputs;
          };

          # checks.default = self'.packages.default;
        };
    };
}
