#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: 0.4.0-alpha
# date: 2025-08-01 20:35:12
parse_number__12_v0() {
    local text=$1
     [ -n "${text}" ] && [ "${text}" -eq "${text}" ] 2>/dev/null ;
    __AS=$?;
if [ $__AS != 0 ]; then
__AF_parse_number12_v0=''
return $__AS
fi
    __AF_parse_number12_v0="${text}";
    return 0
}
file_exists__33_v0() {
    local path=$1
     [ -f "${path}" ] ;
    __AS=$?;
if [ $__AS != 0 ]; then
        __AF_file_exists33_v0=0;
        return 0
fi
    __AF_file_exists33_v0=1;
    return 0
}
file_chmod__39_v0() {
    local path=$1
    local mode=$2
    file_exists__33_v0 "${path}";
    __AF_file_exists33_v0__61_8="$__AF_file_exists33_v0";
    if [ "$__AF_file_exists33_v0__61_8" != 0 ]; then
         chmod "${mode}" "${path}" ;
        __AS=$?
        __AF_file_chmod39_v0=1;
        return 0
fi
    echo "The file ${path} doesn't exist"'!'""
    __AF_file_chmod39_v0=0;
    return 0
}
is_command__93_v0() {
    local command=$1
     [ -x "$(command -v ${command})" ] ;
    __AS=$?;
if [ $__AS != 0 ]; then
        __AF_is_command93_v0=0;
        return 0
fi
    __AF_is_command93_v0=1;
    return 0
}
input_prompt__94_v0() {
    local prompt=$1
     read -p "$prompt" ;
    __AS=$?
    __AF_input_prompt94_v0="$REPLY";
    return 0
}
is_root__98_v0() {
    __AMBER_VAL_0=$( id -u );
    __AS=$?;
    if [ $([ "_${__AMBER_VAL_0}" != "_0" ]; echo $?) != 0 ]; then
        __AF_is_root98_v0=1;
        return 0
fi
    __AF_is_root98_v0=0;
    return 0
}
file_download__135_v0() {
    local url=$1
    local path=$2
    is_command__93_v0 "curl";
    __AF_is_command93_v0__9_9="$__AF_is_command93_v0";
    is_command__93_v0 "wget";
    __AF_is_command93_v0__12_9="$__AF_is_command93_v0";
    is_command__93_v0 "aria2c";
    __AF_is_command93_v0__15_9="$__AF_is_command93_v0";
    if [ "$__AF_is_command93_v0__9_9" != 0 ]; then
         curl -L -o "${path}" "${url}" ;
        __AS=$?
elif [ "$__AF_is_command93_v0__12_9" != 0 ]; then
         wget "${url}" -P "${path}" ;
        __AS=$?
elif [ "$__AF_is_command93_v0__15_9" != 0 ]; then
         aria2c "${url}" -d "${path}" ;
        __AS=$?
else
        __AF_file_download135_v0=0;
        return 0
fi
    __AF_file_download135_v0=1;
    return 0
}
__0_server_url="https://raw.githubusercontent.com/MADMAN-Modding/rlsd/refs/heads/master"
__1_RED="\e[31m"
__2_GREEN="\e[32m"
__3_BLUE="\e[34m"
__4_RESET="\e[0m"
__AMBER_ARRAY_1=("x86_64" "musl" "armv7" "aarch64");
__5_architectures=("${__AMBER_ARRAY_1[@]}")
echo_color__143_v0() {
    local text=$1
    local color=$2
    # I'm not sure why but I need to do the reset on a separate concatenation
    __6_text="${color}${text}"
     echo -e "${__6_text}"${__4_RESET};
    __AS=$?
}
download_rlsd__144_v0() {
    local version=$1
    echo_color__143_v0 "Installing rlsd..." "${__3_BLUE}";
    __AF_echo_color143_v0__23_5="$__AF_echo_color143_v0";
    echo "$__AF_echo_color143_v0__23_5" > /dev/null 2>&1
    file_exists__33_v0 "/usr/bin/rlsd";
    __AF_file_exists33_v0__25_8="$__AF_file_exists33_v0";
    if [ "$__AF_file_exists33_v0__25_8" != 0 ]; then
         sudo rm -f /usr/bin/rlsd ;
        __AS=$?
fi
    file_download__135_v0 "${__0_server_url}/bin/linux/${version}" "/usr/bin/rlsd";
    __AF_file_download135_v0__29_27="$__AF_file_download135_v0";
    local download_result="$__AF_file_download135_v0__29_27"
    file_chmod__39_v0 "/usr/bin/rlsd" "+x";
    __AF_file_chmod39_v0__31_24="$__AF_file_chmod39_v0";
    local chmod_result="$__AF_file_chmod39_v0__31_24"
    __AF_download_rlsd144_v0=$(echo ${download_result} '&&' ${chmod_result} | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
setup__145_v0() {
    local prompt="Please choose one of the following architectures:
"
    i=0;
for arch in "${__5_architectures[@]}"; do
        prompt+="$(echo ${i} '+' 1 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//')) ${arch}
"
    (( i++ )) || true
done
    # This gets the user-input as a number
    input_prompt__94_v0 "${prompt}";
    __AF_input_prompt94_v0__45_18="${__AF_input_prompt94_v0}";
    local choice="${__AF_input_prompt94_v0__45_18}"
    parse_number__12_v0 "${choice}";
    __AS=$?;
if [ $__AS != 0 ]; then
__AF_setup145_v0=''
return $__AS
fi;
    __AF_parse_number12_v0__47_18="$__AF_parse_number12_v0";
    local choice="$__AF_parse_number12_v0__47_18"
    if [ $(echo $(echo ${choice} '==' 0 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') '||' $(echo ${choice} '>' "${#__5_architectures[@]}" | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') != 0 ]; then
        echo_color__143_v0 "Invalid choice" "${__1_RED}";
        __AF_echo_color143_v0__50_9="$__AF_echo_color143_v0";
        echo "$__AF_echo_color143_v0__50_9" > /dev/null 2>&1
        __AF_setup145_v0='';
        return 1
fi
    local architecture="${__5_architectures[$(echo ${choice} '-' 1 | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//')]}"
    echo_color__143_v0 "${architecture} selected" "${__3_BLUE}";
    __AF_echo_color143_v0__56_5="$__AF_echo_color143_v0";
    echo "$__AF_echo_color143_v0__56_5" > /dev/null 2>&1
    # Downloads the selected binary
    download_rlsd__144_v0 "rlsd-${architecture}";
    __AF_download_rlsd144_v0__59_8="$__AF_download_rlsd144_v0";
    if [ "$__AF_download_rlsd144_v0__59_8" != 0 ]; then
        echo_color__143_v0 "Install Succeeded" "${__2_GREEN}";
        __AF_echo_color143_v0__59_46="$__AF_echo_color143_v0";
        echo "$__AF_echo_color143_v0__59_46" > /dev/null 2>&1
else
        echo_color__143_v0 "Install Failed, check the above error message" "${__1_RED}";
        __AF_echo_color143_v0__60_11="$__AF_echo_color143_v0";
        echo "$__AF_echo_color143_v0__60_11" > /dev/null 2>&1
        __AF_setup145_v0='';
        return 1
fi
    __AF_setup145_v0=1;
    return 0
}
declare -r args=("$0" "$@")
    # Checks if the user is root
    is_root__98_v0 ;
    __AF_is_root98_v0__67_8="$__AF_is_root98_v0";
    if [ "$__AF_is_root98_v0__67_8" != 0 ]; then
        echo_color__143_v0 "Root check passed" "${__2_GREEN}";
        __AF_echo_color143_v0__68_9="$__AF_echo_color143_v0";
        echo "$__AF_echo_color143_v0__68_9" > /dev/null 2>&1
        setup__145_v0 ;
        __AS=$?;
        __AF_setup145_v0__70_28="$__AF_setup145_v0";
        result="$__AF_setup145_v0__70_28"
        if [ ${result} != 0 ]; then
            echo_color__143_v0 "Setup Finished" "${__2_GREEN}";
            __AF_echo_color143_v0__72_20="$__AF_echo_color143_v0";
            echo "$__AF_echo_color143_v0__72_20" > /dev/null 2>&1
else
            echo_color__143_v0 "Setup Failed" "${__1_RED}";
            __AF_echo_color143_v0__73_15="$__AF_echo_color143_v0";
            echo "$__AF_echo_color143_v0__73_15" > /dev/null 2>&1
fi
else
        echo_color__143_v0 "Root check failed; please run as root" "${__1_RED}";
        __AF_echo_color143_v0__75_9="$__AF_echo_color143_v0";
        echo "$__AF_echo_color143_v0__75_9" > /dev/null 2>&1
fi
