{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs, ... }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f {
            pkgs = nixpkgs.legacyPackages.${system};
            inherit system;
          }
        );

      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
    {
      packages = forAllSystems (
        { pkgs, ... }:
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            meta.mainProgram = cargoToml.package.name;
          };
        }
        # GTK4 GUI (awww / mpvpaper are Wayland only, so Linux only)
        // pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
          gui = pkgs.rustPlatform.buildRustPackage {
            pname = "${cargoToml.package.name}-gui";
            version = cargoToml.package.version;

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            buildFeatures = [ "gui" ];

            nativeBuildInputs = with pkgs; [
              pkg-config
              wrapGAppsHook4
            ];
            buildInputs = with pkgs; [
              gtk4
              gst_all_1.gst-plugins-base
              gst_all_1.gst-plugins-good
            ];
            meta.mainProgram = "${cargoToml.package.name}-gui";
          };
        }
      );

      apps = forAllSystems (
        { system, ... }:
        {
          default = {
            type = "app";
            program = "${self.packages.${system}.default}/bin/${cargoToml.package.name}";
          };
        }
      );

      devShells = forAllSystems (
        { pkgs, ... }:
        {
          default = pkgs.mkShell {
            packages =
              with pkgs;
              [
                rustc
                cargo
                rust-analyzer
                clippy
                rustfmt
              ]
              ++ lib.optionals stdenv.isLinux [
                pkg-config
                gtk4
              ];
          };
        }
      );
    };
}
