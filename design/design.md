# Command Pattern

```sh
boilerplato my_app --template rust-cli-app

boilerplato my_app --template john/golang-server

boilerplato my_app --template file:../my-custom-template

boilerplato my_app --template https://github.com/rousan/rust-cli-boilerplate.git
``` 

# Install

```sh
bash -c "$(curl -fsSL https://boilerplato.com/install)"
```

# Use Template without installing Boilerplato

```sh
bash -c "$(curl -fsSL https://boilerplato.com/gen)" -- <app_name> <template_name>
```

# Config file

The config file should be: `boilerplato.yml` or `boilerplato.yaml` or `boilerplato.json`

Note: Please find the `boilerplato.yml`(It has the latest format) and `boilerplato.json` for the format.

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
- built-in: https://docs.rs/handlebars/3.0.1/handlebars/#built-in-helpers
- json_str
- concat
- ternary
- color
- replace
- os

# The following In-Built `template variables data` will be available at all handlebars syntax including helpText:
- appName [the app folder name] e.g. `{{ appName }}`

# Available environment variables and substitute variables for post generate commands and help text in boilerplato.yml file:
- All the template data with their exact name
- Running system's envs
- APP_FULL_PATH
- APP_REL_PATH [app folder's relative path based on app generation directory]
- APP_NAME [app folder name]
- TEMPLATE_PATH [Template folder root path]
- TEMPLATE_SOURCE_PATH [Actual template files source path]

> The helpText in boilerplato.yml file supports handlebars and the template data can be used there and the helper functions as well.
> So, the helpText becomes so powerful as:
>   - it can embed environment variables and template data as `helpText: "cd ${appName}"` or `helpText: "echo ${PATH}"`
>   - it supports handlebars syntax as `helpText: "cd {{ appName }}"` or to color a text add `helpText: "{{ color "cd" "red" }} $appName"`



> The following environment variables are available to `defaultValue` attribute for only `string` data type under `data` in `boilerpalto.yml` file:
>   - Running system's envs
>   - APP_FULL_PATH
>   - APP_REL_PATH [app folder's relative path based on app generation directory]
>   - APP_NAME [app folder name]
>   - TEMPLATE_PATH [Template folder root path]
>   - TEMPLATE_SOURCE_PATH [Actual template files source path]
Use it as follows:
```yaml
data:
    - name: packageName
      type: string
      message: "Enter Rust package name: "
      required: false
      defaultValue: $APP_NAME
```