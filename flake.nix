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

        # egui's native window dynamically loads EGL/OpenGL and xkbcommon.
        guiRuntimeDeps = with pkgs; [ libGL libxkbcommon libx11 libxcursor libxi libxrandr ];
        runtimeDeps = libs.transcrust ++ guiRuntimeDeps;
        # The host Mesa driver can require a newer glibc than this flake's
        # binary. Give only the review subprocess a driver from the same pin.
        reviewEglVendor = "${pkgs.mesa}/share/glvnd/egl_vendor.d/50_mesa.json";

        # PATH-wrapped tools (binaries the app shells out to, not libraries)
        # llama-cpp supplies llama-server, which --long uses to run s1-mini as
        # the normalisation pass. Without it on PATH transcrust falls back to the
        # ONNX build silently and loses about 6x on token rate, so it belongs in
        # the wrapper rather than being assumed present.
        runtimeTools = with pkgs; [ wtype dotool ydotool libnotify llama-cpp fuzzel ];

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

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeDeps;
          TRANSCRUST_EGL_VENDOR_LIBRARY_FILENAMES = reviewEglVendor;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };

        # Optional offline model research only; never part of the app runtime.
        devShells.pair-probe = pkgs.mkShell {
          packages = [ (pkgs.python3.withPackages (p: [ p.onnxruntime p.tokenizers p.numpy ])) ];
        };

        packages.default = pkgs.symlinkJoin {
          name = "transcrust";
          paths = [ unwrapped ];
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram $out/bin/transcrust \
              --prefix PATH : "${pkgs.lib.makeBinPath runtimeTools}" \
              --set TRANSCRUST_EGL_VENDOR_LIBRARY_FILENAMES "${reviewEglVendor}" \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath runtimeDeps}"
          '';
        };
      });
}
