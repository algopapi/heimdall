CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    pubkey VARCHAR(44) UNIQUE NOT NULL CHECK (length(pubkey) = 44),
    signature TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_pubkey ON users(pubkey);
CREATE INDEX idx_users_created_at ON users(created_at);
