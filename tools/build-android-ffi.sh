#!/usr/bin/env bash
set -euo pipefail

: "${ANDROID_NDK_HOME:?Set ANDROID_NDK_HOME to the Mopvox NDK}"
: "${ORT_ANDROID_AAR:?Set ORT_ANDROID_AAR to the onnxruntime-android AAR used by Mopvox}"
: "${1:?Usage: bash tools/build-android-ffi.sh OUTPUT_JNI_LIBS_DIR}"

repo=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$1"
output=$(cd "$1" && pwd)
ndk="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
linker="$ndk/aarch64-linux-android30-clang"
export CARGO_TARGET_DIR="$repo/target/android-ffi"
ort_dir="$CARGO_TARGET_DIR/ort-link"
mkdir -p "$ort_dir" "$output/arm64-v8a"
unzip -p "$ORT_ANDROID_AAR" jni/arm64-v8a/libonnxruntime.so > "$ort_dir/libonnxruntime.so"

export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$linker"
export CC_aarch64_linux_android="$linker"
export AR_aarch64_linux_android="$ndk/llvm-ar"
export ORT_LIB_PATH="$ort_dir"
export ORT_PREFER_DYNAMIC_LINK=1
cd "$repo"
cargo rustc --release --locked --target aarch64-linux-android -p transcrust-ffi --features asr,decision --lib \
    -- -C link-arg=-Wl,-z,max-page-size=16384
cp "$CARGO_TARGET_DIR/aarch64-linux-android/release/libtranscrust_ffi.so" "$output/arm64-v8a/"
