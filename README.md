# auria — Agent crate for AURIA Runtime

This repository contains a **production-oriented Rust skeleton** for an `auria` crate designed to act as
the **agent/controller** for an AURIA Runtime deployment.

**AURIA — Autonomous Universal Runtime for Intelligence Assembly**

## What this is

- `auria` is an **agent** that:
  - exposes an API (HTTP) for chat/requests
  - applies policy (tier selection, cost limits, routing preferences)
  - dispatches requests to one or more **Auria Nodes** (AURIA Runtime Core instances)
  - aggregates usage receipts (optional) and forwards them to settlement services

## What this is NOT

- This is not the AURIA Runtime Core itself (execution kernels / expert assembly).
- This skeleton does **compile cleanly**, but leaves integrations (wallet, Base contracts, IPFS, advanced routing)
  as clearly marked TODOs.

## Quickstart

```bash
cargo build
cargo test
cargo run -- --help

# Start the agent
cargo run -- serve --bind 127.0.0.1:8787

# Health check
curl -s http://127.0.0.1:8787/healthz

# Submit a generation request (OpenAI-style-ish)
curl -s http://127.0.0.1:8787/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model":"AURIA:STANDARD","messages":[{"role":"user","content":"Hello Auria"}],"max_tokens":32}'
```

## Configuration

Environment variables (all optional):

- `AURIA_BIND` (default `127.0.0.1:8787`)
- `AURIA_NODE_URLS` comma-separated list (default `http://127.0.0.1:8080`)
- `AURIA_DEFAULT_TIER` one of `NANO|STANDARD|PRO|MAX` (default `STANDARD`)
- `AURIA_MAX_COST_MICROUSDC` (default `0` = unlimited in skeleton)
- `RUST_LOG` (default `info`)

## Deployment

- Dockerfile included
- Example systemd unit included
- GitHub Actions CI included

## License

MIT
