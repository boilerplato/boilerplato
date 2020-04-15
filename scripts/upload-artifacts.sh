#!/bin/bash

set -e

export "$(xargs <.env)"

github-release upload \
    --user boilerplato \
    --repo boilerplato \
    --tag "v$1" \
    --name "boilerplato-v$1-x86_64-apple-darwin.tar.xz" \
    --file "dist/boilerplato-v$1-x86_64-apple-darwin.tar.xz"

github-release upload \
    --user boilerplato \
    --repo boilerplato \
    --tag "v$1" \
    --name "boilerplato-v$1-x86_64-unknown-linux-gnu.tar.xz" \
    --file "dist/boilerplato-v$1-x86_64-unknown-linux-gnu.tar.xz"

github-release upload \
    --user boilerplato \
    --repo boilerplato \
    --tag "v$1" \
    --name "boilerplato-v$1-x86_64-pc-windows-gnu.zip" \
    --file "dist/boilerplato-v$1-x86_64-pc-windows-gnu.zip"

