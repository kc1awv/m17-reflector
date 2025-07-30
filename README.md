# M17 Reflector

This project implements a simple [M17](https://m17project.org/) voice reflector in Rust. It listens for M17 control and stream packets over UDP, forwards voice streams between connected peers and interlinks, and exposes runtime information via an HTTP API and WebSocket.

A small web dashboard is available in `web/` that consumes the API and websocket to display connected peers and live streams.

## Live Implementation

A live implementation of this reflector is provided by KC1AWV and is avaliable as M17-AWV.

You can connect your M17 client program to `m17-awv.kc1awv.net:17000`

The dashboard for M17-AWV is available at [https://m17-awv.kc1awv.net](https://m17-awv.kc1awv.net)


## Dependencies

Major crates used by the application include:

- `tokio` for asynchronous runtime
- `axum` for the HTTP API and WebSocket support
- `clap` for command line parsing
- `serde` for configuration and JSON serialization
- `log` and `env_logger` for logging
- `thiserror` for error management

See `Cargo.toml` for the full list and versions.

## Repository layout

- `src/config.rs` – loads `config.toml` describing the reflector
- `src/callsign.rs` – utilities for M17 callsign encoding/decoding
- `src/crc.rs` – CRC‑16 calculation used by stream packets
- `src/packet.rs` – parsing of control and stream packet formats
- `src/module.rs` and `src/peer.rs` – data structures for modules and connected peers
- `src/reflector.rs` – tracks modules, users, and stream activity
- `src/router.rs` – routes voice stream packets to peers
- `src/control.rs` – handles connection/ping/disconnect control packets
- `src/server.rs` – UDP server loop and keep‑alive task
- `src/api.rs` – REST API endpoints returning stats
- `src/ws.rs` – WebSocket endpoint that pushes live stats snapshots
- `web/` – static HTML/CSS/JS dashboard

## Building

A recent Rust toolchain is required. Build the binary in release mode with

```bash
cargo build --release
```

Run the reflector using a configuration file (defaults to `config.toml`):

```bash
cargo run -- --config config.toml
```

A sample configuration is available as `config.toml.dist`.
Copy it to `config.toml` and adjust fields such as `reflector_name`,
`bind_address`, `modules`, and optional `interlinks` as needed.
The HTTP API and WebSocket listen on port `8080`.

Clients that only wish to monitor traffic may connect using a `LSTN` control
packet instead of `CONN`. They will receive calls routed to their chosen module
but any stream data they transmit will be ignored.

## Using the dashboard

The HTML files under `web/` are completely static. Serve them with any
static web server (for example `python3 -m http.server`) so that the
JavaScript can connect to `ws://<host>:8080/ws` for live updates.

The JSON API is available under `/api/v1/` and provides endpoints for
stats, clients, modules, active streams and recent streams.

## Running as a systemd service

To keep the reflector running in the background you can install it as a
systemd service. Build the release binary and copy it along with a
configuration file:

```bash
cargo build --release
sudo install -Dm755 target/release/m17_reflector /usr/local/bin/m17_reflector
sudo install -Dm644 config.toml /etc/m17-reflector/config.toml
```

An example service unit file is provided under `systemd/m17-reflector.service`.
Copy it to `/etc/systemd/system/` and enable the service:

```bash
sudo cp systemd/m17-reflector.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now m17-reflector.service
```

The service runs under the `m17` user by default and logs using
`env_logger` with `RUST_LOG=info`. Adjust the paths or user in the unit
file as required.
