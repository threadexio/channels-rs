{
  description = "A crate that allows for easy and fast communication between processes, threads and systems.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustVersion = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        formatter = pkgs.nixpkgs-fmt;

        devShells = rec {
          # For writing code.
          # $ nix develop
          dev = pkgs.mkShell {
            packages = with pkgs; [
              rustVersion
              just

              gdb
              lldb
              valgrind
              linuxPackages.perf
              cargo-flamegraph

              python311Packages.virtualenv
            ];

            shellHook = ''
              export PATH="$PWD/bin:$PATH"

              python3 -m venv .python
              source .python/bin/activate
              python -m pip install -r bin/requirements.txt
            '';
          };

          # For editing the artwork of the repo.
          # $ nix develop '.#art'
          art = pkgs.mkShell {
            buildInputs = with pkgs; [
              inkscape
              scour

              # Assets
              orbitron
            ];
          };

          default = dev;
        };
      });
}
