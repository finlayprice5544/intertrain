# Intertrain — Build & Smoke Test Status

**Date:** 2026-10-16
**Build:** cargo build succeeds (17 dead-code warnings, 0 errors)
**Server:** boots on port 8787, RocksDB embedded, ClickHouse optional (falls back to seed)
**Smoke test:** 32/32 passed (0 failures)

---

## Endpoints

### GET Routes

| Endpoint | Status | Notes |
|---|---|---|
| `GET /healthz` | OK (200) | Lightweight health probe for Railway / LB |
| `GET /metrics` | OK (200) | Prometheus-format text/plain |
| `GET /api/_health` | OK (200) | JSON: dataMode, clickhouse status |
| `GET /api/_branding` | OK (200) | JSON: site name, ticker, socials, theme |
| `GET /api-indexer/health` | OK (200) | Seed: indexer height + status |
| `GET /api-indexer/tokens` | OK (200) | Seed: token listing for /tokens/ page |
| `GET /` | OK (200) | Root page (static frontend) |
| `GET /wallet/` | OK (200) | Wallet page (static frontend) |

### POST RPC (JSON-RPC 2.0)

Both `/api/rpc` (devnet) and `/api-mainnet/rpc` (mainnet) share the same dispatch.

| RPC Method | Status | Handler | Notes |
|---|---|---|---|
| `chain_info` | OK | chain service | Chain ID, heights, native asset info |
| `validator_set` | OK | chain service | 4 validators from seed |
| `bridge_status` | OK | chain service | Per-lane bridge status (ETH/ARB/SOL/USDC) |
| `token_list` | OK | chain service | Empty list (no custom tokens created) |
| `asset_list` | OK | chain service | 4 assets: WSK, WUSDC, WETH, WSOL |
| `mna_reserve_status` | OK | chain service | Reserve collateralization, rates, totals |
| `account_get` | OK | chain service | Echoes address, returns balance/nonce |
| `token_evm_address` | OK | chain service | Derives deterministic EVM address from hash |
| `transaction_broadcast` | OK | chain service | Returns simulated tx hash, persists in RocksDB |
| `transaction_status` | OK | chain service | Looks up persisted tx or returns not_found |
| `token_operation_prepare` | OK | chain service | Returns signing_bytes + fee |
| `token_operation_broadcast` | OK | chain service | Returns operation_id, persists in RocksDB |
| `token_operation_status` | OK | chain service | Looks up persisted op or returns not_found |
| `auth_challenge` | OK | auth service | Generates challenge message + nonce, stores in RocksDB |
| `auth_verify` | OK | auth service | Returns session_token (simulation: always succeeds) |
| `mna_quote` | OK | chain service | 1:1 USDC/WSK peg quote |
| `mna_swap_prepare` | OK | chain service | Returns signing_bytes for MNA swap |
| `mna_swap_broadcast` | OK | chain service | Returns operation_id, persists in RocksDB |
| `mna_swap_status` | OK | chain service | Looks up persisted swap or returns not_found |
| `sol_mna_quote` | OK | chain service | SOL->WSK oracle quote (simulated 150:1) |
| `devnet_faucet` | OK | chain service | Credits 10 WSK to address in RocksDB |

### Bridge Auxiliary Methods (via bridge service)

These are handled by the bridge service dispatcher but are not among the 21 "core" wallet RPC methods:

| Method | Status | Notes |
|---|---|---|
| `bridge_lane_detail` | Available | Rich detail per lane (contracts, volumes) |
| `bridge_deposit` | Available | Simulate a bridge deposit event |
| `bridge_event_status` | Available | Look up bridge event by ID |

---

## Data Mode

- **Current mode:** `auto` (live with seed fallback)
- **Seed data loaded:** 24 collections from `seed/*.json`
- **ClickHouse:** Not connected (unavailable); all analytics routes use RocksDB seed
- **RocksDB:** Embedded, always available at `./data/rocks`

All 21 RPC methods work in seed mode. Stateful methods (faucet, transaction_broadcast, token ops, auth, swaps) persist state in RocksDB for the session.

---

## Architecture

- **Dispatch chain:** rpc_handler -> chain service -> auth service -> bridge service -> seed-based dispatch (rpc.rs)
- **No conflicts:** A3 (routes) and A4 (services) both modified rpc.rs cleanly. The services handle stateful methods first; the seed dispatch in rpc.rs acts as fallback for anything services don't claim.
- **Route registration:** Inventory-based auto-mount (7 route groups)

---

## Known Gaps

1. **Rebrand:** No custom branding configured. The `cargo run --bin rebrand` runs but finds no changes because branding.json values match the originals. Set environment variables or edit `config/branding.json` to rebrand.
2. **ClickHouse:** Not available locally. Analytics routes fall back to seed data. For production, set `CLICKHOUSE_URL` etc.
3. **Dead code warnings:** 17 warnings for unused service functions (dexscreener, jupiter, solana, ipfs, http). These services are scaffolded for `DATA_MODE=live` but not yet wired into routes.
4. **Simulation only:** All chain/wallet operations are simulated. No real signing, no real chain interaction. Transaction hashes are deterministic/time-based.
5. **Auth is permissive:** `auth_verify` always succeeds (no Ed25519 verification). This is by design for the simulation backend.
6. **No rate limiting:** Faucet has no cooldown enforcement. The `.env.example` documents `FAUCET_COOLDOWN_SECS` but it is not enforced in code.
7. **Token list empty:** `token_list` returns an empty array. Custom token creation is not yet implemented (would need a `token_create` RPC method).
