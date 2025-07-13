CREATE TABLE transactions (
    id BIGSERIAL PRIMARY KEY,
    pool_id INTEGER NOT NULL REFERENCES pools(id) ON DELETE CASCADE,
    protocol_id INTEGER NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    tx_signature VARCHAR(88) UNIQUE NOT NULL CHECK (length(tx_signature) = 88),
    tx_type VARCHAR(20) NOT NULL CHECK (tx_type IN ('swap', 'add_liquidity', 'remove_liquidity', 'claim_fees')),
    amount_in DECIMAL(30, 8) NOT NULL DEFAULT 0 CHECK (amount_in >= 0),
    amount_out DECIMAL(30, 8) NOT NULL DEFAULT 0 CHECK (amount_out >= 0),
    token_in VARCHAR(44) NOT NULL CHECK (length(token_in) = 44),
    token_out VARCHAR(44) NOT NULL CHECK (length(token_out) = 44),
    price DECIMAL(20, 8) CHECK (price >= 0),
    fee DECIMAL(20, 8) NOT NULL DEFAULT 0 CHECK (fee >= 0),
    slot BIGINT NOT NULL CHECK (slot > 0),
    block_time TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT transactions_different_tokens CHECK (
        tx_type != 'swap' OR token_in != token_out
    )
);

CREATE INDEX idx_transactions_pool_id ON transactions(pool_id);
CREATE INDEX idx_transactions_protocol_id ON transactions(protocol_id);
CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transactions_tx_signature ON transactions(tx_signature);
CREATE INDEX idx_transactions_tx_type ON transactions(tx_type);
CREATE INDEX idx_transactions_block_time ON transactions(block_time DESC);
CREATE INDEX idx_transactions_slot ON transactions(slot DESC);
CREATE INDEX idx_transactions_token_in ON transactions(token_in);
CREATE INDEX idx_transactions_token_out ON transactions(token_out);
CREATE INDEX idx_transactions_pool_time ON transactions(pool_id, block_time DESC);
CREATE INDEX idx_transactions_user_time ON transactions(user_id, block_time DESC);
