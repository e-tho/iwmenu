{
  description = "An interactive menu for managing wifi through iwd.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        mkDate = longDate:
          builtins.substring 0 4 longDate +
          builtins.substring 4 2 longDate +
          builtins.substring 6 2 longDate;
        lastCommitDate =  mkDate (self.lastModifiedDate or "19700101");
        commitHash = self.shortRev or self.dirtyShortRev or "unknown";
        version = "${lastCommitDate}_${commitHash}";
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "iwmenu";
          inherit version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = with pkgs; [
            pkg-config
            openssl
          ];

          cargoBuildFlags = [
            "--release"
          ];

          doCheck = true;
          CARGO_BUILD_INCREMENTAL = "false";
          RUST_BACKTRACE = "full";
          copyLibs = true;

          meta = {
            description = "An interactive menu for managing wifi on Linux.";
            homepage = "https://github.com/e-tho/iwmenu";
            license = pkgs.lib.licenses.gpl3;
            maintainers = [
              {
                github = "e-tho";
              }
            ];
          };
        };

        devShells.default = with pkgs; mkShell {
          buildInputs = [
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
            })
          ];
        };
      }
    );
}
