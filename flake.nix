{
  description = "Guestbook";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
        guestbook = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          buildInputs = with pkgs; [ openssl ];
          nativeBuildInputs = with pkgs; [ pkg-config ];
        };
      in {
        packages.default = guestbook;
        devShells.default = craneLib.devShell {
          packages = with pkgs; [ cargo rustc rust-analyzer ];
        };
      });
}
