# iBark

[![Views](https://hits.seeyoufarm.com/api/count/incr/badge.svg?url=https%3A%2F%2Fgithub.com%2Fuplau%2Fibark&count_bg=%2379C83D&title_bg=%23555555&icon=&icon_color=%23E7E7E7&title=Views&edge_flat=false)](https://hits.seeyoufarm.com)
[![GitHub workflow status](https://github.com/uplau/ibark/actions/workflows/cicd.yaml/badge.svg)](https://github.com/uplau/ibark/actions/workflows/cicd.yaml)
[![GitHub release (with filter)](https://img.shields.io/github/v/release/uplau/ibark)](https://github.com/uplau/ibark/releases/latest)
![GitHub all releases](https://img.shields.io/github/downloads/uplau/ibark/total)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](./LICENSE-MIT)

iBark is a fully featured [Bark](https://github.com/Finb/Bark) cross-platform command line tool written in Rust.

From now on, easily and securely send notifications to your apple devices from the terminal or web interface.

![demo](./.assets/demo.svg?raw=true)

## Table of contents

- [Features](#features)
- [Quick start](#quick-start)
- [Install](#install)
- [Usage](#usage)
- [Configuration](#configuration)
- [Example](#example)
- [Contributing](#contributing)
- [Contributors](#contributors)
- [License](#license)

## Features

- [x] Ease of use in mind
- [x] Shell completion `bash` `elvish` `fish` `powershell` `zsh`
- [x] Choose your `Bark` remote
- [x] Request remote `healthz` `info` `ping`
- [x] Send once notification
- [x] Specifying multiple devices to send
- [x] Support for remote basic-auth
- [x] Support end-to-end encryption
- [ ] `WIP` Web interface
- [ ] `WIP` Send template
- [ ] `WIP` Send scheduler

## Quick start

```bash
# Send once notification
$ your_device_key=""

# Simple
$ ibark send "d://${your_device_key}" -c 't=iBark 💗' -c 'b=Hello 👋\nWorld🌍'

# URL scheme
$ ibark send "d://${your_device_key}" -c 't=iBark 💗' -c 'b=Open github repo' -c 'u=https://github.com/uplau/ibark'

# Icon, available only on iOS15 or later
$ ibark send "d://${your_device_key}" -c 't=iBark 💗' -c 'b=Icon' -c 'i=https://cdn.jsdelivr.net/gh/walkxcode/dashboard-icons@master/png/apple.png'

# See more at https://github.com/Finb/bark-server/blob/master/docs/API_V2.md#push

# Request remote
$ ibark healthz
$ ibark info
$ ibark ping

# Just dump data, will not execute
$ ibark <COMMAND> -D
$ ibark <COMMAND> -DD
```

## Install

Download binaries archive from [Github releases](https://github.com/uplau/ibark/releases), unzip to your `$PATH`.

- For `Unix`, the idiomatic path is `/usr/local/bin`

## Usage

Any time run `ibark <-h|--help>` or `ibark <COMMAND> <-h|--help>`, should be obvious.

> `-h`: short help
> `--help`: verbose help

```bash
iBark is a fully featured Bark cross-platform command line tool written in Rust

Usage: ibark [OPTIONS] [COMMAND]

Commands:
  healthz  Get remote healthz
  info     Get remote info
  ping     Ping remote
  send     Send once notification
  server   [WIP]Web interface
  help     Print this message or the help of the given subcommand(s)

Options:
  -g, --gen <SHELL>         If provided, outputs the completion file for given shell
  -C, --config <PATHS>      Specify configuration files
  -D, --dump...             Just dump data, will not execute
  -R, --remote <REMOTE>     Specify remote
  -U, --user-agent <AGENT>  Specify user-agent
  -h, --help                Print help (see more with '--help')
  -V, --version             Print version
```

## Configuration

### Hierarchical structure

If you do not explicitly specify configuration files with ibark, it will in turn probe for configuration files named `preset.<ini,json,json5,toml,yml,yaml>` in the following dirs:

- global:
  - Windows: `%PUBLIC%\ibark\`
  - Unix: `/etc/ibark/`
- user: `$HOME/.config/ibark/`
- current: `./`

> **Note:** The above file does not have to exist, as it provides sensible defaults.
> If a key is specified in multiple configuration files, the values will be merged together.

> You can run `ibark <COMMAND> <-D|--dump>` to view the final configuration.

### Configuration format

> **Here are a few Special syntax:**

- `remote`: `<http|https>://[<username>:[password]@]<remote>[:port]`
  - `simple`: `https://api.day.app`
  - `port`: `https://api.day.app:65535`
  - `basic-auth`: `https://username:password@hello.world`
- `devices`: `<d|aes>://[<aes_key>:[aes_iv]@]<device_key>[<aes_key_bitlen>/<aes_mode>/<aes_padding>]`
  - `simple`: `d://zd10IOkoPAbkctTRrVPRhe`
  - `aes128cbc`: `aes://3Niw4zYjwOyYr7Bz:zOb8r84B3KrGjs1j@Uz28emqNYwUfmjolArYwyP/128/cbc/pkcs7`
  - `aes192ecb`: `aes://PO9juShj1cxZYkiNf5Bs5jxU:kOOt2GYVcqIc8EeZ@1Oto6FxeC2hTiIfNoNjofG/192/ecb/pkcs7`
  - `aes256ecb`: `aes://sN9iOmq9DniEQYLkQfieKRAZvhMOAjam:A8s7RXBFDNUbS3US@albTJWqsiDe6rjr5CsHCca/256/ecb/pkcs7`

The following is a quick overview of all settings, with `YAML` format:

> Anyone is always welcome to contribute examples in other formats
> [INI]() > [JSON]() > [TOML]()

```yaml
# fallback: https://api.day.app
remote: ...

# fallback: ibark/VERSION
user_agent: ...

devices:
  simple: ...
  aes128cbc: ...
  aes192ecb: ...
  aes256ecb: ...
  awesome_name: ...

# see more at https://github.com/Finb/bark-server/blob/master/docs/API_V2.md#push
# compatible with v1 and v2
# fallback:
#   a: 1       # or autocopy or automaticallycopy, any non-null value will be formatted as '1' to follow the upstream api
#   l: active  # or level, any null value will be formatted as 'active' to follow the upstream api
# receives first character or full key, not case sensitive, here are a few exceptions
contexts:
  bdg: 1 # or badge
  cat: "" # or category, reserved field, no use yet
  isa: 1 # or isarchive, any non-null value will be formatted as '1' to follow the upstream api

# fallback: 10
limit_conn: ...
```

## Example

> Anyone is always welcome to contribute examples

### View final configuration

```bash
$ cat preset.yaml

devices:
  awesome_name: ...

# -D   will dump related configuration
# -DD  will dump ⬆️ and configuration source
$ ibark send awesome_name -D
$ ibark send awesome_name -DD
$ ibark send d://... aes://... -D
$ ibark healthz -D
$ ibark info -D
$ ibark ping -DD
```

### Specify bark remote

```bash
$ cat preset.yaml

remote: https://hello.world

$ ibark send d://...
$ ibark send d://... -R 'https://hi.world'

# basic-auth
$ ibark send d://... -R 'https://username:password@hi.world'
```

### Send to multiple devices

```bash
$ cat preset.yaml

devices:
  simple: ...
  awesome_name: ...

$ ibark send simple awesome_name d://.... aes://...
```

### End-to-End encryption

```bash
$ cat preset.yaml

devices:
  aes128cbc: aes://3Niw4zYjwOyYr7Bz:zOb8r84B3KrGjs1j@Uz28emqNYwUfmjolArYwyP/128/cbc/pkcs7
  aes192ecb: aes://PO9juShj1cxZYkiNf5Bs5jxU:kOOt2GYVcqIc8EeZ@1Oto6FxeC2hTiIfNoNjofG/192/ecb/pkcs7
  aes256ecb: aes://sN9iOmq9DniEQYLkQfieKRAZvhMOAjam:A8s7RXBFDNUbS3US@albTJWqsiDe6rjr5CsHCca/256/ecb/pkcs7

$ ibark send aes128cbc aes192ecb aes256ecb d://...
```

### Fixed contexts

```bash
$ cat preset.yaml

contexts:
  icon: ...
  group: ...
  sound: ...

devices:
  awesome_name: ...

# will be used when contexts are not explicitly specified
$ ibark send awesome_name -D

# if you do not need these presets, you need to override them explicitly
$ ibark send awesome_name -c 'i=' -c 'g=other_group' -c 's=' -D
```

### Shell completion

```bash
# for other shells, you should be able to find relevant information from the Internet
# bash idiomatic path: /etc/bash_completion.d/
$ ibark -g bash > /etc/bash_completion.d/completion_ibark
$ source /etc/bash_completion.d/completion_ibark
```

## Contributing

This repository was created with [rust-template](https://github.com/uplau/rust-template).

See the [contributing guidelines](./CONTRIBUTING.md) for more information.

## Contributors

<a href="https://github.com/uplau/ibark/graphs/contributors">
<img src="https://contrib.rocks/image?repo=uplau/ibark&max=400&columns=20" />
</a>

## License

iBark is licensed under either of the following, at your option:

- [MIT License](./LICENSE-MIT)
- [Apache-2.0 License](./LICENSE-APACHE)
