#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: 0.4.0-alpha
# date: 2025-08-01 12:42:32

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
echo_color__46_v0 "Building Linux..." "${__2_BLUE}";
__AF_echo_color46_v0__15_1="$__AF_echo_color46_v0";
echo "$__AF_echo_color46_v0__15_1" > /dev/null 2>&1
 cargo build --release ;
__AS=$?;
if [ $__AS != 0 ]; then
    echo_color__46_v0 "Linux build failed" "${__0_RED}";
    __AF_echo_color46_v0__18_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__18_5" > /dev/null 2>&1
fi
echo_color__46_v0 "Building Linux-musl..." "${__2_BLUE}";
__AF_echo_color46_v0__21_1="$__AF_echo_color46_v0";
echo "$__AF_echo_color46_v0__21_1" > /dev/null 2>&1
 rustup target add x86_64-unknown-linux-musl; cargo build --release --target x86_64-unknown-linux-musl ;
__AS=$?;
if [ $__AS != 0 ]; then
    echo_color__46_v0 "Linux-musl build failed" "${__0_RED}";
    __AF_echo_color46_v0__24_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__24_5" > /dev/null 2>&1
fi
# Copy files to bin folder
dir_exists__32_v0 "target";
__AF_dir_exists32_v0__28_4="$__AF_dir_exists32_v0";
if [ "$__AF_dir_exists32_v0__28_4" != 0 ]; then
    linux_build_path="target/release/rlsd"
    linux_musl_build_path="target/x86_64-unknown-linux-musl/release/rlsd"
    echo_color__46_v0 "'target' dir found...
Copying files..." "${__2_BLUE}";
    __AF_echo_color46_v0__32_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__32_5" > /dev/null 2>&1
     cp ${linux_build_path} bin/linux/rlsd ;
    __AS=$?
     cp ${linux_musl_build_path} bin/linux/rlsd-musl ;
    __AS=$?
    echo_color__46_v0 "Copy Completed" "${__1_GREEN}";
    __AF_echo_color46_v0__37_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__37_5" > /dev/null 2>&1
else
    echo_color__46_v0 "'target' dir not found, make sure you are running this script in the project root" "${__0_RED}";
    __AF_echo_color46_v0__39_5="$__AF_echo_color46_v0";
    echo "$__AF_echo_color46_v0__39_5" > /dev/null 2>&1
fi
