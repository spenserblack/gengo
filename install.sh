#!/bin/sh
set -e

OS_NAME=$(uname -s)
ARCH_NAME=$(uname -m)
OS=""
ARCH=""
INSTALL_DIR="/usr/local/bin"
NO_RELEASE_ASSET=""

echo "Installing gengo to $INSTALL_DIR"
echo "This script will activate sudo to install to $INSTALL_DIR"
sudo echo "sudo activated"

if [ "$OS_NAME" = "Linux" ]; then
	OS="Linux"
elif [ "$OS_NAME" = "Darwin" ]; then
	OS="macOS"
else
	NO_RELEASE_ASSET="true"
	echo "There is no Unix release for this OS: $OS_NAME" >&2
fi

if [ "$ARCH_NAME" != "x86_64" ]; then
	NO_RELEASE_ASSET="true"
	echo "There is no release for this architecture: $ARCH_NAME" >&2
else
	ARCH="X64"
fi

if [ "$NO_RELEASE_ASSET" ]; then
	exit 1
fi

curl -sSL "https://github.com/spenserblack/gengo/releases/latest/download/gengo-$OS-$ARCH.tar.gz" | sudo tar -C "$INSTALL_DIR" -xzf -
echo "successfully installed gengo"
