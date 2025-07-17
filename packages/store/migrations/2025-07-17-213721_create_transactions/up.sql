-- Your SQL goes here
CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    signature BYTEA NOT NULL,
    is_vote BOOLEAN NOT NULL,
    slot BIGINT NOT NULL,
    idx BIGINT NOT NULL
);

CREATE TABLE sanitized_transactions (
    id SERIAL PRIMARY KEY,
    transaction_id INTEGER REFERENCES transactions(id),
    message_hash BYTEA NOT NULL,
    is_simple_vote_transaction BOOLEAN NOT NULL
);

CREATE TABLE transaction_signatures (
    id SERIAL PRIMARY KEY,
    sanitized_transaction_id INTEGER REFERENCES sanitized_transactions(id),
    signature BYTEA NOT NULL
);

CREATE TABLE transaction_status_meta (
    id SERIAL PRIMARY KEY,
    transaction_id INTEGER REFERENCES transactions(id),
    is_status_err BOOLEAN NOT NULL,
    error_info TEXT,
    fee BIGINT NOT NULL
);

CREATE TABLE transaction_pre_balances (
    id SERIAL PRIMARY KEY,
    status_meta_id INTEGER REFERENCES transaction_status_meta(id),
    balance BIGINT NOT NULL
);

CREATE TABLE transaction_post_balances (
    id SERIAL PRIMARY KEY,
    status_meta_id INTEGER REFERENCES transaction_status_meta(id),
    balance BIGINT NOT NULL
);

CREATE TABLE transaction_log_messages (
    id SERIAL PRIMARY KEY,
    status_meta_id INTEGER REFERENCES transaction_status_meta(id),
    log_message TEXT NOT NULL
);

CREATE TABLE transaction_inner_instructions (
    id SERIAL PRIMARY KEY,
    status_meta_id INTEGER REFERENCES transaction_status_meta(id),
    idx INTEGER NOT NULL
);

CREATE TABLE transaction_inner_instruction (
    id SERIAL PRIMARY KEY,
    inner_instructions_id INTEGER REFERENCES transaction_inner_instructions(id),
    stack_height INTEGER,
    program_id_index INTEGER NOT NULL,
    data BYTEA NOT NULL
);

CREATE TABLE transaction_pre_token_balances (
    id SERIAL PRIMARY KEY,
    status_meta_id INTEGER REFERENCES transaction_status_meta(id),
    account_index INTEGER NOT NULL,
    mint TEXT NOT NULL,
    owner TEXT,
    ui_amount DOUBLE PRECISION,
    decimals INTEGER,
    amount TEXT,
    ui_amount_string TEXT
);

CREATE TABLE transaction_post_token_balances (
    id SERIAL PRIMARY KEY,
    status_meta_id INTEGER REFERENCES transaction_status_meta(id),
    account_index INTEGER NOT NULL,
    mint TEXT NOT NULL,
    owner TEXT,
    ui_amount DOUBLE PRECISION,
    decimals INTEGER,
    amount TEXT,
    ui_amount_string TEXT
);

CREATE TABLE transaction_rewards (
    id SERIAL PRIMARY KEY,
    status_meta_id INTEGER REFERENCES transaction_status_meta(id),
    pubkey TEXT NOT NULL,
    lamports BIGINT NOT NULL,
    post_balance BIGINT NOT NULL,
    reward_type INTEGER NOT NULL,
    commission INTEGER
);