-- ClickHouse wallet-specific tables.
-- Extends 001_init.sql with tables for wallet operations:
-- transactions, token operations, swaps, bridge events, auth events.
-- Apply:  clickhouse-client < migrations/clickhouse/002_wallet_tables.sql

-- ──────────────────────────────────────────────
-- Native transfers (transaction_broadcast / transaction_status)
-- ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS app.transactions
(
    ts              DateTime64(3) DEFAULT now64(3),
    hash            String,
    chain_id        LowCardinality(String),
    height          UInt64 DEFAULT 0,
    status          LowCardinality(String),       -- 'pending' | 'confirmed' | 'not_found'
    from_address    String,
    to_address      String,
    amount          UInt64 DEFAULT 0,              -- in microWSK
    fee             UInt64 DEFAULT 0,
    nonce           UInt64 DEFAULT 0,
    memo            String DEFAULT '',
    network         LowCardinality(String) DEFAULT 'mainnet'  -- 'mainnet' | 'devnet'
)
ENGINE = MergeTree
ORDER BY (chain_id, ts, hash);

-- ──────────────────────────────────────────────
-- Token operations (Create / Transfer / Mint / Burn / Freeze)
-- ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS app.token_operations
(
    ts              DateTime64(3) DEFAULT now64(3),
    operation_id    String,
    chain_id        LowCardinality(String),
    height          UInt64 DEFAULT 0,
    status          LowCardinality(String),       -- 'pending' | 'confirmed' | 'not_found'
    kind            LowCardinality(String),       -- 'Create' | 'Transfer' | 'Mint' | 'Burn' | 'Freeze'
    token_id        String,
    from_address    String,
    to_address      String DEFAULT '',
    amount          UInt64 DEFAULT 0,
    fee             UInt64 DEFAULT 0,
    nonce           UInt64 DEFAULT 0,
    name            String DEFAULT '',
    symbol          String DEFAULT '',
    decimals        UInt8 DEFAULT 0,
    network         LowCardinality(String) DEFAULT 'mainnet'
)
ENGINE = MergeTree
ORDER BY (chain_id, kind, ts, operation_id);

-- ──────────────────────────────────────────────
-- MNA reserve swaps (MintMna / RedeemMna)
-- ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS app.swaps
(
    ts                  DateTime64(3) DEFAULT now64(3),
    operation_id        String,
    chain_id            LowCardinality(String),
    height              UInt64 DEFAULT 0,
    status              LowCardinality(String),
    kind                LowCardinality(String),   -- 'MintMna' | 'RedeemMna'
    from_address        String,
    collateral_asset    String DEFAULT '',
    amount_usdc         UInt64 DEFAULT 0,          -- in micro (6 decimals)
    amount_mna          UInt64 DEFAULT 0,          -- in microWSK
    fee                 UInt64 DEFAULT 0,
    rate                String DEFAULT '',
    network             LowCardinality(String) DEFAULT 'mainnet'
)
ENGINE = MergeTree
ORDER BY (chain_id, kind, ts, operation_id);

-- ──────────────────────────────────────────────
-- Bridge events (Ethereum USDC, Arbitrum WSK, Solana SOL/USDC)
-- ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS app.bridge_events
(
    ts                  DateTime64(3) DEFAULT now64(3),
    event_id            String,                    -- deposit tx hash or bridge nonce
    lane                LowCardinality(String),    -- 'ethereum' | 'arbitrum' | 'solana' | 'solana_usdc'
    direction           LowCardinality(String),    -- 'deposit' | 'withdrawal'
    source_chain        LowCardinality(String),    -- 'Ethereum Sepolia' | 'Arbitrum One' | 'Solana devnet'
    source_tx           String DEFAULT '',
    source_address      String DEFAULT '',
    dest_address        String,                    -- wsk1... on Intertrain
    asset_symbol        LowCardinality(String),    -- 'USDC' | 'WSK' | 'WSOL' | 'WETH'
    amount              UInt64 DEFAULT 0,
    decimals            UInt8 DEFAULT 6,
    confirmations       UInt32 DEFAULT 0,
    status              LowCardinality(String),    -- 'pending' | 'confirmed' | 'failed'
    bridge_contract     String DEFAULT '',
    network             LowCardinality(String) DEFAULT 'mainnet'
)
ENGINE = MergeTree
ORDER BY (lane, ts, event_id);

-- ──────────────────────────────────────────────
-- Auth challenge/verify events (wallet ownership proof)
-- ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS app.auth_events
(
    ts              DateTime64(3) DEFAULT now64(3),
    address         String,
    domain          String,
    nonce           String,
    action          LowCardinality(String),       -- 'challenge' | 'verify'
    success         UInt8 DEFAULT 1,
    session_token   String DEFAULT '',
    ip              String DEFAULT '',
    network         LowCardinality(String) DEFAULT 'mainnet'
)
ENGINE = MergeTree
ORDER BY (address, ts);

-- ──────────────────────────────────────────────
-- Faucet drips (devnet only)
-- ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS app.faucet_drips
(
    ts              DateTime64(3) DEFAULT now64(3),
    address         String,
    amount          UInt64 DEFAULT 0,
    hash            String DEFAULT '',
    network         LowCardinality(String) DEFAULT 'devnet'
)
ENGINE = MergeTree
ORDER BY (address, ts);
