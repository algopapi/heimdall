CREATE TABLE orders (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pool_id INTEGER NOT NULL REFERENCES pools(id) ON DELETE CASCADE,
    protocol_id INTEGER NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    order_type VARCHAR(20) NOT NULL CHECK (order_type IN ('market', 'limit', 'stop_loss', 'take_profit')),
    side VARCHAR(4) NOT NULL CHECK (side IN ('buy', 'sell')),
    price DECIMAL(20, 8) CHECK (price >= 0),
    amount DECIMAL(30, 8) NOT NULL CHECK (amount > 0),
    filled_amount DECIMAL(30, 8) NOT NULL DEFAULT 0 CHECK (filled_amount >= 0 AND filled_amount <= amount),
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'filled', 'cancelled', 'expired', 'rejected')),
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    CONSTRAINT orders_limit_price_required CHECK (
        order_type != 'limit' OR price IS NOT NULL
    ),
    
    CONSTRAINT orders_valid_expiry CHECK (
        status != 'pending' OR expires_at IS NULL OR expires_at > now()
    )
);

CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_pool_id ON orders(pool_id);
CREATE INDEX idx_orders_protocol_id ON orders(protocol_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_side ON orders(side);
CREATE INDEX idx_orders_order_type ON orders(order_type);
CREATE INDEX idx_orders_created_at ON orders(created_at DESC);
CREATE INDEX idx_orders_updated_at ON orders(updated_at DESC);
CREATE INDEX idx_orders_user_status ON orders(user_id, status);
CREATE INDEX idx_orders_pool_status ON orders(pool_id, status);
CREATE INDEX idx_orders_expires_at ON orders(expires_at) WHERE status = 'pending';