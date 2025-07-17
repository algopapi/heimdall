-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS transaction_rewards;
DROP TABLE IF EXISTS transaction_post_token_balances;
DROP TABLE IF EXISTS transaction_pre_token_balances;
DROP TABLE IF EXISTS transaction_inner_instruction;
DROP TABLE IF EXISTS transaction_inner_instructions;
DROP TABLE IF EXISTS transaction_log_messages;
DROP TABLE IF EXISTS transaction_post_balances;
DROP TABLE IF EXISTS transaction_pre_balances;
DROP TABLE IF EXISTS transaction_status_meta;
DROP TABLE IF EXISTS transaction_signatures;
DROP TABLE IF EXISTS sanitized_transactions;
DROP TABLE IF EXISTS transactions;
