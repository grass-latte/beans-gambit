{
  description = "A Nix-flake-based Rust development environment";

    inputs = {
      nixpkgs = {
        type = "github";
        owner = "NixOS";
        repo = "nixpkgs";
        ref = "nixos-25.11";
      };
      flake-utils.url = "github:numtide/flake-utils";
    };

  outputs = { self, nixpkgs, flake-utils }: # , rust-overlay
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
#          overlays = [ rust-overlay.overlays.default ];
            config.permittedInsecurePackages = [
            "dotnet-sdk-6.0.428"
            "dotnet-runtime-6.0.36"
          ];
        };

        packages = with pkgs; [
          fastchess
          cutechess
          stockfish
        ];

        nativeBuildPackages = with pkgs; [
            dotnetCorePackages.sdk_6_0
            dotnet-runtime_6
        ];

        libraries = with pkgs; [

        ];


      in {
        devShells.default = pkgs.mkShell {
          buildInputs = packages;

          nativeBuildInputs = nativeBuildPackages;

          DOTNET_BIN = "${pkgs.dotnetCorePackages.sdk_6_0}/bin/dotnet";

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