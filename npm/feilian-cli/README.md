# feilian-cli

Unofficial community CLI for Feilian/CorpLink. This package is not affiliated with or endorsed by Feilian.

The npm package installs only the native binary matching the current platform:

- macOS: Apple Silicon and Intel
- Linux: arm64 and x64
- Windows: x64

## Install

```sh
npm install --global feilian-cli
```

## Configure and run

The first run asks only for your company identifier and optional enterprise account/email. Login is fixed to QR code, followed by Feilian app push verification when the server supports it. The CLI saves `feilian-cli.config.json` in your user home directory, then immediately starts the connection:

```sh
feilian-cli
```

An explicit configuration path remains supported:

```sh
feilian-cli /path/to/config.json
```

Check npm for a newer release:

```sh
feilian-cli --check-update
```

Normal runs also perform a short, non-fatal update check. Update with:

```sh
npm install --global feilian-cli@latest
```

The source code, build instructions, and GPL license are available at https://github.com/huhuanming/corplink-rs.

## Security

The generated config is created with user-only permissions on Unix. Do not publish passwords, OTP seeds, session cookies, VPN tokens, private keys, or production account details.

## License

GPL-2.0-or-later.
