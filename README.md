# 𝖥𝖫∀𝖬∫ - Flexiformal Annotation Management System

## Setup

Dowload the latest version for your operating system at [releases](https://github.com/KWARC/FLAMS/releases/tag/latest), unzip
somewhere and run the `flams` executable.

Afterwards, open `http://localhost:8095` to see the 𝖥𝖫∀𝖬∫ *dashboard* (the address/port can be changed; see
below). If the port is already taken, it will increase the port number until it finds a free one
and print a corresponding message.

You can close `flams` again using `CTRL+C`.

## Configuration

Most settings are configurable via one of three ways:
1. A command line argument,
2. An environment variable,
3. An entry in a settings `.toml` file.

One notable exception is the toml file to use for other settings, the (absolute or relative)
path of which is provided via the `-c` or `--config-file` command line arguments, and `--lsp`, which
runs `flams` as an LSP language server. If no
explicit settings file is provided, `flams` will at startup look for a `settings.toml` in the
same directory as the executable and use that, if existent.
CL arguments have priority over environment variables, which have priority over the `.toml`
settings file, which have priority over default values

| Setting | CL argument | `.toml` entry | environment variable | value | default |
| --- |  --- | --- | --- | --- | --- |
| MathHub directories | `-m`, `--mathhubs` | `mathhubs` | `MATHHUB` | comma-separated list of directory paths | `~/.mathhub` |
| Debug logging | `-d`, `--debug` | `debug` | `FLAMS_DEBUG` | `true`/`false` | `false` |
| Log directory | `-l`, `--log-dir` | `log_dir` | `FLAMS_LOG_DIR` | directory path | `~/.flams/log` |
| Temporary directory (used for e.g. sandboxed build queues) | `--temp-dir` | `temp_dir` | `FLAMS_TEMP_DIR` | directory path | `~/.flams/tmp` |
| (User) Database File | `--db`  | `database` | `FLAMS_DATABASE` | file path | `~/.flams/users.sqlite`
| | | `[server]` | | | |
| (Internal) Server IP | `--ip` | `ip` | `FLAMS_IP` | IPv4 address | `127.0.0.1` |
| Server port | `--port` | `port` | `FLAMS_PORT` | port number | `8095` |
| External URL (at which the server is reachable from other IPs) | `--external-url` | `external_url` | `FLAMS_EXTERNAL_URL` | URL | None / `<ip>:<port>`
| Admin Pasword (If set, makes the server run in "public mode") | `-a`, `--admin-pwd` | `admin_pwd` | `FLAMS_ADMIN_PWD` | (string) | (None) |
| | | `[buildqueue]` | | | |
| Number of maximal simultaneous build jobs | `-t`, `--threads` | `num_threads` | `FLAMS_NUM_THREADS` | (positive integer) | number of CPU cores / 2 |
| | | `[gitlab]` | | | |
| URL of a to-be-managed gitlab instance | `--gitlab-url` | `url` | `FLAMS_GITLAB_URL` | URL | (None) |
| Gitlab App Id for this flams instance | `--gitlab-app-id` | `app_id` | `FLAMS_GITLAB_APP_ID` | string | (None) |
| Gitlab App secret for this flams instance | `--gitlab-app-secret` | `app_secret` | `FLAMS_GITLAB_APP_SECRET` | string | (None) |
| OAuth redirect base URL (usually = external URL above) | `--gitlab-redicrect-url`| `redirect_url` | `FLAMS_GITLAB_REDIRECT_URL` | url | (None)

## User Manual
TODO

## Compile From Source

```sh
cargo install cargo-make
rustup target add wasm32-unknown-unknown
cargo make
```

Use `cargo make dev` for a (faster) development build.

Code Documentation hosted [here](https://kwarc.github.io/FLAMS).
