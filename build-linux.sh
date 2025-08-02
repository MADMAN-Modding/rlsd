#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: 0.4.0-alpha
# date: 2025-08-01 19:29:35

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
    __4_text="${color}${text}"
     echo -e "${__4_text}"${__3_RESET};
    __AS=$?
}
echo_color__46_v0 "Building Linux-x86_64..." "${__2_BLUE}";
__AF_echo_color46_v0__15_1="$__AF_echo_color46_v0";
echo "$__AF_echo_color46_v0__15_1" > /dev/null 2>&1
 sleep 1 ;
__AS=$?
 cargo build --release ;
__AS=$?;
if [ $__AS != 0 ]; then
    echo_color__46_v0 "Linux build failed" "${__0_RED}";
    __AF_echo_color46_v0__20_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__20_5" > /dev/null 2>&1
fi
__AMBER_ARRAY_0=("x86_64-unknown-linux-musl" "armv7-unknown-linux-gnueabihf" "aarch64-unknown-linux-gnu");
__5_architectures=("${__AMBER_ARRAY_0[@]}")
__AMBER_ARRAY_1=("musl" "armv7" "aarch64");
__6_arch_names=("${__AMBER_ARRAY_1[@]}")
i=0;
for arch in "${__5_architectures[@]}"; do
    echo_color__46_v0 "Building Linux-${__6_arch_names[${i}]}..." "${__2_BLUE}";
    __AF_echo_color46_v0__28_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__28_5" > /dev/null 2>&1
     sleep 1 ;
    __AS=$?
     cross build --target ${arch} --release ;
    __AS=$?;
if [ $__AS != 0 ]; then
        echo_color__46_v0 "${__6_arch_names[${i}]} build failed" "${__0_RED}";
        __AF_echo_color46_v0__33_9="$__AF_echo_color46_v0";
        echo "$__AF_echo_color46_v0__33_9" > /dev/null 2>&1
fi
    (( i++ )) || true
done
# Copy files to bin folder
dir_exists__32_v0 "target";
__AF_dir_exists32_v0__38_4="$__AF_dir_exists32_v0";
if [ "$__AF_dir_exists32_v0__38_4" != 0 ]; then
    linux_build_path="target/release/rlsd"
    echo_color__46_v0 "'target' dir found...
Copying files..." "${__2_BLUE}";
    __AF_echo_color46_v0__41_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__41_5" > /dev/null 2>&1
     cp ${linux_build_path} bin/linux/rlsd ;
    __AS=$?
    i=0;
for arch in "${__5_architectures[@]}"; do
        build_path="target/${arch}/release/rlsd"
         cp ${build_path} bin/linux/rlsd-${__6_arch_names[${i}]} ;
        __AS=$?
    (( i++ )) || true
done
    echo_color__46_v0 "Copy Completed" "${__1_GREEN}";
    __AF_echo_color46_v0__50_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__50_5" > /dev/null 2>&1
else
    echo_color__46_v0 "'target' dir not found, make sure you are running this script in the project root" "${__0_RED}";
    __AF_echo_color46_v0__52_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__52_5" > /dev/null 2>&1
fi
