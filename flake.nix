{
  description = "Launcher-driven Wi-Fi manager for Linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "iwmenu";
          version = self.shortRev or self.dirtyShortRev or "unknown";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          doCheck = true;
          CARGO_BUILD_INCREMENTAL = "false";
          RUST_BACKTRACE = "full";

          meta = {
            description = "Launcher-driven Wi-Fi manager for Linux";
            homepage = "https://github.com/e-tho/iwmenu";
            license = pkgs.lib.licenses.gpl3Plus;
            maintainers = [
              {
                github = "e-tho";
              }
            ];
            mainProgram = "iwmenu";
          };
        };

        devShells.default =
          with pkgs;
          mkShell {
            nativeBuildInputs = [
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" ];
              })
            ];

            inherit (self.packages.${system}.default) CARGO_BUILD_INCREMENTAL RUST_BACKTRACE;
          };
      }
    );
}
