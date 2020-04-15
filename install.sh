#!/bin/bash

set -e

echo "$1"

LINUX_ARTIFACT="https://github.com/boilerplato/boilerplato/releases/download/v$1/boilerplato-v$1-x86_64-unknown-linux-gnu.tar.xz"
MACOS_ARTIFACT="https://github.com/boilerplato/boilerplato/releases/download/v$1/boilerplato-v$1-x86_64-apple-darwin.tar.xz"

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  curl -fsSL "$LINUX_ARTIFACT" --output /tmp/boilerplato.output.tar.xz
  tar -xf /tmp/boilerplato.output.tar.xz -C /usr/local/bin/
  rm -rf /tmp/boilerplato.output.tar.xz
elif [[ "$OSTYPE" == "darwin"* ]]; then
  curl -fsSL "$MACOS_ARTIFACT" --output /tmp/boilerplato.output.tar.xz
  tar -xf /tmp/boilerplato.output.tar.xz -C /usr/local/bin/
  rm -rf /tmp/boilerplato.output.tar.xz
else
  echo "Unknown target"
fi
