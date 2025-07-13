CREATE TABLE pools (
    id SERIAL PRIMARY KEY,
    protocol_id INTEGER NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    pool_pubkey VARCHAR(44) UNIQUE NOT NULL CHECK (length(pool_pubkey) = 44),
    base_mint VARCHAR(44) NOT NULL CHECK (length(base_mint) = 44),
    quote_mint VARCHAR(44) NOT NULL CHECK (length(quote_mint) = 44),
    base_decimals SMALLINT NOT NULL CHECK (base_decimals >= 0 AND base_decimals <= 18),
    quote_decimals SMALLINT NOT NULL CHECK (quote_decimals >= 0 AND quote_decimals <= 18),
    fee_numerator BIGINT NOT NULL DEFAULT 0 CHECK (fee_numerator >= 0),
    fee_denominator BIGINT NOT NULL DEFAULT 1000000 CHECK (fee_denominator > 0),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT pools_different_mints CHECK (base_mint != quote_mint)
);

CREATE INDEX idx_pools_protocol_id ON pools(protocol_id);
CREATE INDEX idx_pools_pool_pubkey ON pools(pool_pubkey);
CREATE INDEX idx_pools_base_mint ON pools(base_mint);
CREATE INDEX idx_pools_quote_mint ON pools(quote_mint);
CREATE INDEX idx_pools_is_active ON pools(is_active);
CREATE INDEX idx_pools_created_at ON pools(created_at);
CREATE INDEX idx_pools_mint_pair ON pools(base_mint, quote_mint);
