-- Your SQL goes here
CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    slot BIGINT NOT NULL,
    pubkey BYTEA NOT NULL,
    lamports BIGINT NOT NULL,
    owner BYTEA NOT NULL,
    executable BOOLEAN NOT NULL,
    rent_epoch BIGINT NOT NULL,
    data BYTEA,
    write_version BIGINT NOT NULL,
    txn_signature BYTEA
);