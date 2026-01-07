{
  description = "A Nix-flake-based Rust development environment";

    inputs = {
      nixpkgs = {
        type = "github";
        owner = "NixOS";
        repo = "nixpkgs";
        ref = "nixos-25.11";
      };
#      rust-overlay = {
#        url = "github:oxalica/rust-overlay";
#        inputs.nixpkgs.follows = "nixpkgs";
#      };
      flake-utils.url = "github:numtide/flake-utils";
    };

  outputs = { self, nixpkgs, flake-utils }: # , rust-overlay
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
#          overlays = [ rust-overlay.overlays.default ];
        };

        packages = with pkgs; [
          fastchess
          cutechess
          stockfish
        ];

        nativeBuildPackages = with pkgs; [

        ];

        libraries = with pkgs; [

        ];


      in {
        devShells.default = pkgs.mkShell {
          buildInputs = packages;

          nativeBuildInputs = nativeBuildPackages;

          env = {

          };

          shellHook = with pkgs; ''
            export LD_LIBRARY_PATH="${
              lib.makeLibraryPath libraries
            }:$LD_LIBRARY_PATH"
          '';
        };
      });

}