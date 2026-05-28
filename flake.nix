{
  description = "transcrust — voice-to-text audio CLI — migrated";

  # To activate:
  #   cp flake.nix flake.nix.bak && cp flake.nix.proposed flake.nix
  #   nix flake update config
  # To revert:
  #   cp flake.nix.bak flake.nix && rm flake.nix.bak

  inputs = {
    config.url = "github:jaycee1285/config";
    nixpkgs.follows = "config/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, config, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        libs = config.lib.runtimeLibs pkgs;

        src = builtins.path {
          path = ./.;
          name = "transcrust-source";
          filter = path: type:
            let base = builtins.baseNameOf path; in
              !(base == ".git" || base == "result" || base == "target"
                || pkgs.lib.hasSuffix ".tar.xz" base);
        };

        runtimeDeps = libs.transcrust;

        # PATH-wrapped tools (binaries the app shells out to, not libraries)
        runtimeTools = with pkgs; [ wtype dotool ydotool libnotify ];

        unwrapped = pkgs.rustPlatform.buildRustPackage {
          pname = "transcrust-unwrapped";
          version = "0.1.0";
          inherit src;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [ pkg-config cmake git ];
          buildInputs = runtimeDeps;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };
      in
      {
        libs.declared = {
          categories = [ "transcrust" ];
          local = [ "cmake" "llvmPackages.libclang" "wtype" "dotool" "ydotool" ];
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustc cargo rust-analyzer clippy rustfmt pkg-config cmake
          ];
          buildInputs = runtimeDeps ++ runtimeTools;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };

        packages.default = pkgs.symlinkJoin {
          name = "transcrust";
          paths = [ unwrapped ];
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram $out/bin/transcrust \
              --prefix PATH : "${pkgs.lib.makeBinPath runtimeTools}"
          '';
        };
      });
}
