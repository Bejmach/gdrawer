{
  description = "gesture drawing app flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    nixpkgs,
    naersk,
  }: let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
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
    packages.x86_64-linux.default = naerskLib.buildPackage {
      src = "./.";
    };

    devShells."x86_64-linux".default = pkgs.mkShell {
      buildInputs = with pkgs; [
        cargo
        rustc
        rust-analyzer
        clippy
        rustfmt
      ];
      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${nixpkgs.lib.makeLibraryPath dlopenLibraries}";
    };
  };
}
