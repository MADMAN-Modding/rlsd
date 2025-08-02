# R.L.S.D.

<h1>Welcome to
<ul>
<li>Retro</li>
<li>Looking</li>
<li>Statistics</li>
<li>Display</li>
</ul>
</h1>

<p>The purpose of this project is for devices to send statistics to a host machine that will display the information on a retro looking TUI. I started this project because I thought it'd be fun.</p>

# Setup

The script was built using Amber, please install the following package: <a href="https://archlinux.org/packages/extra/x86_64/bc/" target="_blank">`bc`</a>

The script requires root access as it will place the binary in /usr/lib/rlsd, create a user, and create a systemd service

Run the following commands:

        # Downloads the script
        curl -o rlsd-install.sh https://raw.githubusercontent.com/MADMAN-Modding/rlsd/refs/heads/master/install-scripts/linux/setup.sh
        
        # Makes it executable
        chmod +x rlsd-install.sh

        # Run the script as root
        sudo ./rlsd-install.sh

        # Remove the script
        rm rlsd-install.sh

# Building

<h3>Dependencies</h3>
Rust is needed to compile this project, grab it <a href="https://www.rust-lang.org/tools/install" target="_blank">here</a>

<h4>Arch</h4>

    // Install the dependencies needed
    sudo pacman -S --needed \
        base-devel \
        pkgconf \
        openssl \
        sqlite \
        libudev \
        cmake \
        zlib

<h4>Fedora</h4>

    // Install the dependencies needed
    sudo dnf install -y \
        gcc \
        pkgconf-pkg-config \
        openssl-devel \
        sqlite-devel \
        libudev-devel \
        cmake \
        make \
        zlib-devel \ 
        git

<h4>Debian/Ubuntu<h4>

    // Install the dependencies needed
    sudo apt update && sudo apt install -y \
        build-essential \
        pkg-config \
        libssl-dev \
        libsqlite3-dev \
        libudev-dev \
        cmake \
        zlib1g-dev


<h3>Clone the project<h3>

    git clone https://github.com/MADMAN-Modding/rlsd.git

<h3>Building</h3>

<h5>Linux</h5>
To build the default Linux configuration for the Rust project

    cd rlsd
    
    cargo build --release

<h5>Linux Musl</h5>
If you run into glibc version issues do the following

    cd rsld

    rustup target add x86_64-unknown-linux-musl
    
    cargo build --release --target x86_64-unknown-linux-musl

<h3>Running</h3>

If you run into issues running the project, try using Linux Musl

<h5>Linux</h5>

    // Make the binary executable
    chmod +x target/releases/rlsd

    // Run the binary    
    ./target/release/rlsd --help

<h5>Linux Musl</h5>

    // Change directory to the musl release
    cd target/x86_64-unknown-linux-musl/release    

    // Make the binary executable
    chmod +x rlsd

    // Run the binary
    ./rlsd --help