{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
      in rec {
        # `nix build`
        packages.stegfile = naersk-lib.buildPackage {
          pname = "stegfile";
          root = ./.;
        };
        defaultPackage = packages.stegfile;

        # `nix run`
        apps.stegfile = flake-utils.lib.mkApp { drv = packages.stegfile; };
        defaultApp = apps.stegfile;

        # `nix develop`
        devShell =
          pkgs.mkShell { nativeBuildInputs = with pkgs; [ rustc cargo ]; };
      });
}
