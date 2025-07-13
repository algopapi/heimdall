CREATE TABLE user_swaps (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tx_id BIGINT NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT user_swaps_unique_user_tx UNIQUE (user_id, tx_id)
);

CREATE INDEX idx_user_swaps_user_id ON user_swaps(user_id);
CREATE INDEX idx_user_swaps_tx_id ON user_swaps(tx_id);
CREATE INDEX idx_user_swaps_created_at ON user_swaps(created_at DESC);
CREATE INDEX idx_user_swaps_user_time ON user_swaps(user_id, created_at DESC);