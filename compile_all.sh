#!/bin/bash

CURRENT_VER=$(head -n3 Cargo.toml | grep version | cut -f2 -d'=' | cut -f2 -d\")

# all platforms binary
cargo b --release --target aarch64-apple-darwin
cargo b --release --target x86_64-apple-darwin
cargo b --release --target x86_64-pc-windows-gnu
cargo b --release --target aarch64-unknown-linux-gnu
cargo b --release --target x86_64-unknown-linux-gnu

# remove existing files
rm -rf tmp
# make the folder again
mkdir -p tmp

# copy files to the tmp folder
# win
cp target/x86_64-pc-windows-gnu/release/whois.exe tmp/whois_x86-64.exe
# macos
cp target/aarch64-apple-darwin/release/whois tmp/whois_macos_aarch64
cp target/x86_64-apple-darwin/release/whois tmp/whois_macos_x86-64
# linux
cp target/aarch64-unknown-linux-gnu/release/whois tmp/whois_linux_aarch64
cp target/x86_64-unknown-linux-gnu/release/whois tmp/whois_linux_x86-64

# create the new zip files
cd tmp
zip -9r bigdomaindata-whois"${CURRENT_VER}"-windows.zip whois_x86-64.exe
zip -9r bigdomaindata-whois"${CURRENT_VER}"-macos.zip whois_macos_aarch64 whois_macos_x86-64
zip -9r bigdomaindata-whois"${CURRENT_VER}"-linux.zip whois_linux_aarch64 whois_linux_x86-64
cd ..

# delete the tmp files
rm -f tmp/whois_x86-64.exe tmp/whois_macos_aarch64 tmp/whois_macos_x86-64 tmp/whois_linux_aarch64 tmp/whois_linux_x86-64