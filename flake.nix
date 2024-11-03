{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk.url = "github:nix-community/naersk";
    treefmt-nix.url = "github:numtide/treefmt-nix";
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
            pkg-config
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

          naersk = pkgs.callPackage inputs.naersk { };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.fenix.overlays.default
            ];
          };

          treefmt.config = {
            projectRootFile = "flake.nix";
            programs = {
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
            };
          };

          packages.default = naersk.buildPackage {
            src = ./.;
            inherit buildInputs nativeBuildInputs LD_LIBRARY_PATH;
          };

          devShells.default = pkgs.mkShell {
            inherit buildInputs nativeBuildInputs LD_LIBRARY_PATH;
          };
        };
    };
}

/*
  flake-utils.lib.eachSystem systems (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};

        buildInputs = with pkgs; [
          udev alsa-lib vulkan-loader
          xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr
          libxkbcommon wayland
        ];
        nativeBuildInputs = with pkgs; [
          rustc
          cargo
          pkg-config
        ];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          src = ./.;
          inherit buildInputs nativeBuildInputs LD_LIBRARY_PATH;
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs LD_LIBRARY_PATH;
        };

        checks = {
          default = defaultPackage;
        };
      }
  );
  }
*/
