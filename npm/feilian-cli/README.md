# feilian-cli

Unofficial community CLI for Feilian/CorpLink. This package is not affiliated with or endorsed by Feilian.

The initial npm release supports macOS on Apple Silicon (`darwin-arm64`). It bundles a native executable; Node.js is only used by npm to install the command.

## Install

```sh
npm install --global feilian-cli
```

## Run

Pass an explicit configuration file:

```sh
feilian-cli /path/to/config.json
```

The source code, build instructions, and GPL license are available at https://github.com/huhuanming/corplink-rs.

## Security

Do not put passwords, OTP seeds, session cookies, VPN tokens, private keys, or production account details in a public repository or npm package.

## License

GPL-2.0-or-later.
