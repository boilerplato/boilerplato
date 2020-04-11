#!/bin/bash

set -e

LINUX_ARTIFACT="https://github.com/rousan/boilerplato/releases/download/v1.0.0/boilerplato-v1.0.0-x86_64-unknown-linux-gnu.tar.xz"
MACOS_ARTIFACT="https://github.com/rousan/boilerplato/releases/download/v1.0.0/boilerplato-v1.0.0-x86_64-apple-darwin.tar.xz"

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  curl -fsSL "$LINUX_ARTIFACT" --output /tmp/boilerplato.output.tar.xz
  tar -xf /tmp/boilerplato.output.tar.xz -C /usr/local/bin/
  rm -rf /tmp/boilerplato.output.tar.xz
elif [[ "$OSTYPE" == "darwin"* ]]; then
  curl -fsSL "$MACOS_ARTIFACT" --output /tmp/boilerplato.output.tar.xz
  tar -xf /tmp/boilerplato.output.tar.xz -C /usr/local/bin/
  rm -rf /tmp/boilerplato.output.tar.xz
elif [[ "$OSTYPE" == "freebsd"* ]]; then
  echo "Freebsd artifacts not found"
else
  echo "Unknown target"
fi
