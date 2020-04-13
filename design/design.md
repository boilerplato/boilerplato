# Command Pattern

```sh
boilerplato my_app --template rust-cli-app

boilerplato my_app --template john/golang-server

boilerplato my_app --template file:../my-custom-template

boilerplato my_app --template https://github.com/rousan/rust-cli-boilerplate.git
``` 

# Config file

The config file should be: `boilerplato.yml` or `boilerplato.json`

Note: Please find the `boilerplato.yml` and `boilerplato.json` for the format.

Supported Data Types:
- string
- number
- bool
- array[string]
- array[number]
- enum[string]
- enum[number]
- semver

# Template Engine

Currently, only `handlebars` is supported.

# Template Engine: Handlebars

These are the built-in handlebars helper functions will be provided while parsing: