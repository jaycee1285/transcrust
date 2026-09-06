{
  description = "transcrust — VibeVoice ASR ONNX export toolchain (Python, throwaway)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # PyPI wheels (torch, onnxruntime) are manylinux binaries: they need a
        # real glibc C++ runtime on LD_LIBRARY_PATH, which a pure Nix shell does
        # not provide. This is the standard uv-on-NixOS escape hatch — nothing
        # here is built from source.
        wheelLibs = with pkgs; [
          stdenv.cc.cc.lib
          zlib
          glib
          libGL
          openssl
        ];
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [ python312 uv git curl ffmpeg ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath wheelLibs;

          shellHook = ''
            export UV_PYTHON=${pkgs.python312}/bin/python3.12
            export UV_PROJECT_ENVIRONMENT="$PWD/.venv"
            echo "vibevoice-export shell — run: ./export.sh"
          '';
        };
      });
}
