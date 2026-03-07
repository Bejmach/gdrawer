{
  description = "gesture drawing app flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    naersk,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        naerskLib = pkgs.callPackage naersk {};
        dlopenLibraries = with pkgs; [
          libxkbcommon

          # GPU backend
          vulkan-loader
          # libGL

          # Window system
          wayland
          # xorg.libX11
          # xorg.libXcursor
          # xorg.libXi
        ];
      in {
        packages.default = naerskLib.buildPackage {
          src = ./.;

          buildInputs = [pkgs.makeWrapper];

          postInstall = ''
            wrapProgram $out/bin/gdrawer \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath dlopenLibraries}"
          '';
        };
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
          ];
          env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${nixpkgs.lib.makeLibraryPath dlopenLibraries}";
        };
      }
    );
}
