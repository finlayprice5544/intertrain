# {{SITE_NAME}} — generated site backend (Rust)

Synthesized backend for the cloned frontend in the sibling `../frontend/` folder.
**Axum + ClickHouse + embedded RocksDB.** Run everything from this `backend/` dir.

## Run
```bash
cp .env.example .env            # optional — boots without it
docker compose up -d clickhouse # optional — analytics; RocksDB works without it
cargo run --bin rebrand         # push your CA/socials into the frontend
cargo run                       # http://localhost:8787
```

## Prerequisites
- **Rust** (stable) — https://rustup.rs
- **LLVM/Clang** — required to build `rust-rocksdb` (bindgen). Windows: `winget install LLVM.LLVM` then set `LIBCLANG_PATH` to `C:\Program Files\LLVM\bin`. Also needs **VS Build Tools** (MSVC) for the linker.
- **Docker** (optional) — only for the local ClickHouse in `docker-compose.yml`.

## Change your branding (CA / socials / name)
Edit **`config/branding.json`** (or set env vars in `.env` — env wins):
`contractAddress`, `socials.twitter|github|discord|telegram|website`, `siteName`, `ticker`.
Then `cargo run --bin rebrand` applies them to the cloned frontend in `public/`.
API responses read branding live via `src/branding.rs`.

## Data layer
- **RocksDB** (embedded, always on): KV + state + cache + seed collections
  (`seed/<name>.json` → key `seed:<name>`). This is why the app boots offline.
- **ClickHouse** (optional): analytical/event/time-series data (feeds, revenue,
  rewards, trades). Unreachable → routes fall back to seed. Schema in
  `migrations/clickhouse/`. Generic query helper: `db::clickhouse::query_json`.
- `DATA_MODE=seed|live|auto` controls seed vs live external calls.

## API keys (`.env`) — all optional
| var | what | where |
|---|---|---|
| `CLICKHOUSE_URL` | analytics store | docker compose / ClickHouse Cloud |
| `HELIUS_API_KEY` or `SOLANA_RPC_URL` | Solana RPC (read-only) | dev.helius.xyz |
| `DEXSCREENER_BASE` | prices/pairs | no key |
| `JUPITER_BASE` | swap price/quote | no key on lite |
| `BIRDEYE_API_KEY` | price fallback | birdeye.so |
| `PINATA_JWT` | pin metadata | pinata.cloud |

## Add an endpoint
1. `src/routes/<name>.rs` — copy `example_coins.rs` (return the captured envelope).
2. add `pub mod <name>;` to `src/routes/mod.rs`. Auto-mounts via `inventory`.

## Structure
```
../frontend/             cloned frontend (served as-is; sibling folder, PUBLIC_DIR)
src/
  main.rs                axum server (routes + static + spa fallback)
  config.rs              env + config/branding.json
  state.rs               AppState (rocks, clickhouse, http)
  db/{rocks,clickhouse,seed}.rs
  services/{dexscreener,jupiter,solana,ipfs,http}.rs
  routes/{mod,meta,example_coins}.rs   one file per endpoint
  branding.rs            inject CA/socials into responses
  bin/{rebrand,smoke}.rs
seed/*.json              seeded from captured responses
migrations/clickhouse/   table DDL
config/                  branding.json, replacements.json, endpoints.json
docker-compose.yml       local ClickHouse
```

## Note on scope
Your own project scaffold. Swap/launch execution is **simulated**
(`src/services/solana.rs`), not real. Set your own CA + socials; don't deploy
under another project's identity.
