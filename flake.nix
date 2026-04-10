{
  description = "Guestbook";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
        templateFilter = path: _type: builtins.match ".*templates/.*" path != null;
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (templateFilter path type) || (craneLib.filterCargoSources path type);
        };
        guestbook = craneLib.buildPackage {
          inherit src;
          buildInputs = with pkgs; [ openssl ];
          nativeBuildInputs = with pkgs; [ pkg-config ];
        };
      in {
        packages.default = guestbook;
        devShells.default = craneLib.devShell {
          packages = with pkgs; [ cargo rustc rust-analyzer ];
        };
      })) // {
      nixosModules.default = ./module.nix;
    };
}
