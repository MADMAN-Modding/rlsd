#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: 0.4.0-alpha
# date: 2025-08-01 20:20:11

dir_exists__32_v0() {
    local path=$1
     [ -d "${path}" ] ;
    __AS=$?;
if [ $__AS != 0 ]; then
        __AF_dir_exists32_v0=0;
        return 0
fi
    __AF_dir_exists32_v0=1;
    return 0
}
__0_RED="\e[31m"
__1_GREEN="\e[32m"
__2_BLUE="\e[34m"
__3_RESET="\e[0m"
echo_color__46_v0() {
    local text=$1
    local color=$2
    # I'm not sure why but I need to do the reset on a separate concatenation
    __6_text="${color}${text}"
     echo -e "${__6_text}"${__3_RESET};
    __AS=$?
}
# List of target names
__AMBER_ARRAY_0=("x86_64-unknown-linux-gnu" "x86_64-unknown-linux-musl" "armv7-unknown-linux-gnueabihf" "aarch64-unknown-linux-gnu");
__4_architectures=("${__AMBER_ARRAY_0[@]}")
# List of names for each target in the same order
__AMBER_ARRAY_1=("x86_64" "musl" "armv7" "aarch64");
__5_arch_names=("${__AMBER_ARRAY_1[@]}")
# Builds each architecture
i=0;
for arch in "${__4_architectures[@]}"; do
    echo_color__46_v0 "Building Linux-${__5_arch_names[${i}]}..." "${__2_BLUE}";
    __AF_echo_color46_v0__23_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__23_5" > /dev/null 2>&1
     sleep 1 ;
    __AS=$?
     cross build --target ${arch} --release ;
    __AS=$?;
if [ $__AS != 0 ]; then
        echo_color__46_v0 "${__5_arch_names[${i}]} build failed" "${__0_RED}";
        __AF_echo_color46_v0__28_9="$__AF_echo_color46_v0";
        echo "$__AF_echo_color46_v0__28_9" > /dev/null 2>&1
fi
    (( i++ )) || true
done
# Copy files to bin folder
dir_exists__32_v0 "target";
__AF_dir_exists32_v0__33_4="$__AF_dir_exists32_v0";
if [ "$__AF_dir_exists32_v0__33_4" != 0 ]; then
    i=0;
for arch in "${__4_architectures[@]}"; do
        build_path="target/${arch}/release/rlsd"
         cp ${build_path} bin/linux/rlsd-${__5_arch_names[${i}]} ;
        __AS=$?;
if [ $__AS != 0 ]; then
            echo_color__46_v0 "Failed to copy rlsd-${__5_arch_names[${i}]}" "${__0_RED}";
            __AF_echo_color46_v0__38_13="$__AF_echo_color46_v0";
            echo "$__AF_echo_color46_v0__38_13" > /dev/null 2>&1
fi
    (( i++ )) || true
done
    echo_color__46_v0 "Copy Completed" "${__1_GREEN}";
    __AF_echo_color46_v0__42_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__42_5" > /dev/null 2>&1
else
    echo_color__46_v0 "'target' dir not found, make sure you are running this script in the project root" "${__0_RED}";
    __AF_echo_color46_v0__44_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__44_5" > /dev/null 2>&1
fi
