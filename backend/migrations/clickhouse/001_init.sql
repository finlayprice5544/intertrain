-- ClickHouse schema for analytical / event / time-series data.
-- Runs automatically on first `docker compose up` (mounted into initdb.d),
-- or apply manually:  clickhouse-client < migrations/clickhouse/001_init.sql
-- The generator adapts/extends these tables to the site's captured data.

CREATE DATABASE IF NOT EXISTS app;

-- Generic event stream (analytics_events, activity, etc.). Columnar + fast range scans.
CREATE TABLE IF NOT EXISTS app.events
(
    ts          DateTime64(3) DEFAULT now64(3),
    kind        LowCardinality(String),      -- 'activity' | 'trade' | 'launch' | ...
    actor       String,                      -- wallet / user id
    subject     String,                      -- mint / desk id / etc.
    amount      Float64 DEFAULT 0,
    meta        String DEFAULT '{}'          -- JSON blob for extra fields
)
ENGINE = MergeTree
ORDER BY (kind, ts);

-- Example aggregate source: revenue / rewards rows.
CREATE TABLE IF NOT EXISTS app.revenue
(
    day         Date,
    source      LowCardinality(String),
    amount_usd  Float64
)
ENGINE = MergeTree
ORDER BY (source, day);
