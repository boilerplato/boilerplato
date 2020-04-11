#!/bin/bash

set -e

if [ ! -f .env ]; then
    export $(cat .env | xargs)
fi

github-release upload \
    --user boilerplato \
    --repo boilerplato \
    --tag v1.0.0 \
    --name "boilerplato-v1.0.0-x86_64-apple-darwin.tar.xz" \
    --file dist/boilerplato-v1.0.0-x86_64-apple-darwin.tar.xz

github-release upload \
    --user boilerplato \
    --repo boilerplato \
    --tag v1.0.0 \
    --name "boilerplato-v1.0.0-x86_64-apple-darwin.tar.xz" \
    --file dist/boilerplato-v1.0.0-x86_64-apple-darwin.tar.xz

github-release upload \
    --user boilerplato \
    --repo boilerplato \
    --tag v1.0.0 \
    --name "boilerplato-v1.0.0-x86_64-unknown-linux-gnu.tar.xz" \
    --file dist/boilerplato-v1.0.0-x86_64-unknown-linux-gnu.tar.xz
