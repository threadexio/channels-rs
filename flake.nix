{
  description = "A crate that allows for easy and fast communication between processes, threads and systems.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; }; in {
        formatter = pkgs.nixpkgs-fmt;

        devShells = rec {
          # For writing code.
          # $ nix develop
          dev = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustup

              # For tools/header.py
              python312
              virtualenv
            ];

            shellHook = ''
              				virtualenv .python
              				source .python/bin/activate
              				python -m pip install -r tools/requirements.txt
              				'';
          };

          # For editing the artwork of the repo.
          # $ nix develop '.#art'
          art = pkgs.mkShell {
            buildInputs = with pkgs; [ inkscape scour ];
          };

          default = dev;
        };
      });
}
