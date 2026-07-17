-- Idempotency support for checkout
CREATE TABLE idempotency_keys (
    key VARCHAR(255) PRIMARY KEY,
    response_status INTEGER NOT NULL,
    response_body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Auto-cleanup after 24 hours
CREATE INDEX idx_idempotency_created_at ON idempotency_keys(created_at);
