#!/usr/bin/env bash
# Written in [Amber](https://amber-lang.com/)
# version: 0.4.0-alpha
# date: 2025-08-01 12:09:01
lowercase__10_v0() {
    local text=$1
    __AMBER_VAL_0=$( echo "${text}" | tr '[:upper:]' '[:lower:]' );
    __AS=$?;
    __AF_lowercase10_v0="${__AMBER_VAL_0}";
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
input_confirm__96_v0() {
    local prompt=$1
    local default_yes=$2
    local choice_default=$(if [ ${default_yes} != 0 ]; then echo " [\x1b[1mY/\x1b[0mn]"; else echo " [y/\x1b[1mN\x1b[0m]"; fi)
             printf "\x1b[1m${prompt}\x1b[0m${choice_default}" ;
        __AS=$?
         read -s -n 1 ;
        __AS=$?
         printf "
" ;
        __AS=$?
    __AMBER_VAL_1=$( echo $REPLY );
    __AS=$?;
    lowercase__10_v0 "${__AMBER_VAL_1}";
    __AF_lowercase10_v0__90_18="${__AF_lowercase10_v0}";
    local result="${__AF_lowercase10_v0__90_18}"
    __AF_input_confirm96_v0=$(echo $([ "_${result}" != "_y" ]; echo $?) '||' $(echo $([ "_${result}" != "_" ]; echo $?) '&&' ${default_yes} | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//') | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
is_root__98_v0() {
    __AMBER_VAL_2=$( id -u );
    __AS=$?;
    if [ $([ "_${__AMBER_VAL_2}" != "_0" ]; echo $?) != 0 ]; then
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
__0_server_url="https://madman-modding.github.io/rlsd"
__1_RED="\e[31m"
__2_GREEN="\e[32m"
__3_BLUE="\e[34m"
__4_RESET="\e[0m"
echo_color__141_v0() {
    local text=$1
    local color=$2
    # I'm not sure why but I need to do the reset on a separate concatenation
    __5_text="${color}${text}"
     echo -e "${__5_text}"${__4_RESET};
    __AS=$?
}
download_rlsd__142_v0() {
    local version=$1
    echo_color__141_v0 "Installing rlsd..." "${__3_BLUE}";
    __AF_echo_color141_v0__20_5="$__AF_echo_color141_v0";
    echo "$__AF_echo_color141_v0__20_5" > /dev/null 2>&1
    file_download__135_v0 "${__0_server_url}/bin/linux/${version}" "/usr/bin/rlsd";
    __AF_file_download135_v0__22_27="$__AF_file_download135_v0";
    local download_result="$__AF_file_download135_v0__22_27"
    file_chmod__39_v0 "/usr/bin/rlsd" "+x";
    __AF_file_chmod39_v0__24_24="$__AF_file_chmod39_v0";
    local chmod_result="$__AF_file_chmod39_v0__24_24"
    __AF_download_rlsd142_v0=$(echo ${download_result} '&&' ${chmod_result} | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
download_services__143_v0() {
    echo_color__141_v0 "Installing services..." "${__3_BLUE}";
    __AF_echo_color141_v0__30_5="$__AF_echo_color141_v0";
    echo "$__AF_echo_color141_v0__30_5" > /dev/null 2>&1
    local service_dir="/etc/systemd/system"
    # Download client service
    echo "services/linux/rlsd-server.service" > /dev/null 2>&1
    file_download__135_v0 "${__0_server_url}/services/linux/rlsd-client.service" "${service_dir}/rlsd-client.service";
    __AF_file_download135_v0__36_25="$__AF_file_download135_v0";
    local client_result="$__AF_file_download135_v0__36_25"
    # Download server service
    file_download__135_v0 "${__0_server_url}/services/linux/rlsd-server.service" "${service_dir}/rlsd-server.service";
    __AF_file_download135_v0__39_25="$__AF_file_download135_v0";
    local server_result="$__AF_file_download135_v0__39_25"
    # Return true if both succeed
    __AF_download_services143_v0=$(echo ${client_result} '&&' ${server_result} | bc -l | sed '/\./ s/\.\{0,1\}0\{1,\}$//');
    return 0
}
setup__144_v0() {
    # If the user answers yes, rsld-musl is selected, otherwise rlsd is selected
    input_confirm__96_v0 "Download the Linux-musl version? (Recommended if you are having glibc errors)" 0;
    __AF_input_confirm96_v0__48_18="$__AF_input_confirm96_v0";
    local choice=$(if [ "$__AF_input_confirm96_v0__48_18" != 0 ]; then echo "rlsd-musl"; else echo "rlsd"; fi)
    echo_color__141_v0 "${choice} selected" "${__3_BLUE}";
    __AF_echo_color141_v0__52_5="$__AF_echo_color141_v0";
    echo "$__AF_echo_color141_v0__52_5" > /dev/null 2>&1
    # Downloads the selected binary
    download_rlsd__142_v0 "${choice}";
    __AF_download_rlsd142_v0__55_8="$__AF_download_rlsd142_v0";
    if [ "$__AF_download_rlsd142_v0__55_8" != 0 ]; then
        echo_color__141_v0 "Install Succeeded" "${__2_GREEN}";
        __AF_echo_color141_v0__55_31="$__AF_echo_color141_v0";
        echo "$__AF_echo_color141_v0__55_31" > /dev/null 2>&1
else
        echo_color__141_v0 "Install Failed, check the above error message" "${__1_RED}";
        __AF_echo_color141_v0__56_11="$__AF_echo_color141_v0";
        echo "$__AF_echo_color141_v0__56_11" > /dev/null 2>&1
        __AF_setup144_v0='';
        return 1
fi
    # Install services
    input_confirm__96_v0 "${__3_BLUE}Install systemd services?${__4_RESET}" 1;
    __AF_input_confirm96_v0__59_18="$__AF_input_confirm96_v0";
    local choice="$__AF_input_confirm96_v0__59_18"
    if [ ${choice} != 0 ]; then
        download_services__143_v0 ;
        __AF_download_services143_v0__62_12="$__AF_download_services143_v0";
        if [ "$__AF_download_services143_v0__62_12" != 0 ]; then
            echo_color__141_v0 "Install Succeeded" "${__2_GREEN}";
            __AF_echo_color141_v0__62_33="$__AF_echo_color141_v0";
            echo "$__AF_echo_color141_v0__62_33" > /dev/null 2>&1
else
            echo_color__141_v0 "Install Failed, check the above error message" "${__1_RED}";
            __AF_echo_color141_v0__63_15="$__AF_echo_color141_v0";
            echo "$__AF_echo_color141_v0__63_15" > /dev/null 2>&1
fi
fi
    __AF_setup144_v0=1;
    return 0
}
declare -r args=("$0" "$@")
    # Checks if the user is root
    is_root__98_v0 ;
    __AF_is_root98_v0__71_8="$__AF_is_root98_v0";
    if [ "$__AF_is_root98_v0__71_8" != 0 ]; then
        echo_color__141_v0 "Root check passed" "${__2_GREEN}";
        __AF_echo_color141_v0__72_9="$__AF_echo_color141_v0";
        echo "$__AF_echo_color141_v0__72_9" > /dev/null 2>&1
        setup__144_v0 ;
        __AS=$?;
        __AF_setup144_v0__74_28="$__AF_setup144_v0";
        result="$__AF_setup144_v0__74_28"
        if [ ${result} != 0 ]; then
            echo_color__141_v0 "Setup Finished" "${__2_GREEN}";
            __AF_echo_color141_v0__76_20="$__AF_echo_color141_v0";
            echo "$__AF_echo_color141_v0__76_20" > /dev/null 2>&1
else
            echo_color__141_v0 "Setup Failed" "${__1_RED}";
            __AF_echo_color141_v0__77_15="$__AF_echo_color141_v0";
            echo "$__AF_echo_color141_v0__77_15" > /dev/null 2>&1
fi
else
        echo_color__141_v0 "Root check failed; please run as root" "${__1_RED}";
        __AF_echo_color141_v0__79_9="$__AF_echo_color141_v0";
        echo "$__AF_echo_color141_v0__79_9" > /dev/null 2>&1
fi
