#!/bin/bash

set -e

LINUX_ARTIFACT="https://github.com/boilerplato/boilerplato/releases/download/v$1/boilerplato-v$1-x86_64-unknown-linux-gnu.tar.gz"
MACOS_ARTIFACT="https://github.com/boilerplato/boilerplato/releases/download/v$1/boilerplato-v$1-x86_64-apple-darwin.tar.gz"

PWD="$(pwd)"
INSTALLATION_DIR="/usr/local/bin"
CLONE_FILE_NAME="boilerplato.output.tar.gz"
CLONE_DIR="/tmp"
CLONE_PATH="${CLONE_DIR}/${CLONE_FILE_NAME}"

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  echo "Boilerplato v${1}"
  echo "Downlaoding artifact from ${LINUX_ARTIFACT}..."
  curl -fsSL "${LINUX_ARTIFACT}" --output "${CLONE_PATH}"
  cd "${CLONE_DIR}"
  echo "Extracting..."
  tar -xvf "${CLONE_FILE_NAME}"
  echo "Installing..."
  mv boilerplato "${INSTALLATION_DIR}/boilerplato"
  echo "Installed at ${INSTALLATION_DIR}/boilerplato"
  rm -rf "${CLONE_FILE_NAME}"
  cd "${PWD}"
elif [[ "$OSTYPE" == "darwin"* ]]; then
  echo "Boilerplato v${1}"
  echo "Downlaoding artifact from ${MACOS_ARTIFACT}..."
  curl -fsSL "${MACOS_ARTIFACT}" --output "${CLONE_PATH}"
  cd "${CLONE_DIR}"
  echo "Extracting..."
  tar -xvf "${CLONE_FILE_NAME}"
  echo "Installing..."
  mv boilerplato "${INSTALLATION_DIR}/boilerplato"
  echo "Installed at ${INSTALLATION_DIR}/boilerplato"
  rm -rf "${CLONE_FILE_NAME}"
  cd "${PWD}"
else
  echo "Sorry! No build found for the current operating system."
  echo "Please raise an issue here: https://github.com/boilerplato/boilerplato/issues/new"
fi
