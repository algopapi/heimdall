// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Int4,
        slot -> Int8,
        pubkey -> Bytea,
        lamports -> Int8,
        owner -> Bytea,
        executable -> Bool,
        rent_epoch -> Int8,
        data -> Nullable<Bytea>,
        write_version -> Int8,
        txn_signature -> Nullable<Bytea>,
    }
}

diesel::table! {
    sanitized_transactions (id) {
        id -> Int4,
        transaction_id -> Nullable<Int4>,
        message_hash -> Bytea,
        is_simple_vote_transaction -> Bool,
    }
}

diesel::table! {
    slots (id) {
        id -> Int4,
        slot -> Int8,
        parent -> Nullable<Int8>,
        status -> Int4,
    }
}

diesel::table! {
    transaction_inner_instruction (id) {
        id -> Int4,
        inner_instructions_id -> Nullable<Int4>,
        stack_height -> Nullable<Int4>,
        program_id_index -> Int4,
        data -> Bytea,
    }
}

diesel::table! {
    transaction_inner_instructions (id) {
        id -> Int4,
        status_meta_id -> Nullable<Int4>,
        idx -> Int4,
    }
}

diesel::table! {
    transaction_log_messages (id) {
        id -> Int4,
        status_meta_id -> Nullable<Int4>,
        log_message -> Text,
    }
}

diesel::table! {
    transaction_post_balances (id) {
        id -> Int4,
        status_meta_id -> Nullable<Int4>,
        balance -> Int8,
    }
}

diesel::table! {
    transaction_post_token_balances (id) {
        id -> Int4,
        status_meta_id -> Nullable<Int4>,
        account_index -> Int4,
        mint -> Text,
        owner -> Nullable<Text>,
        ui_amount -> Nullable<Float8>,
        decimals -> Nullable<Int4>,
        amount -> Nullable<Text>,
        ui_amount_string -> Nullable<Text>,
    }
}

diesel::table! {
    transaction_pre_balances (id) {
        id -> Int4,
        status_meta_id -> Nullable<Int4>,
        balance -> Int8,
    }
}

diesel::table! {
    transaction_pre_token_balances (id) {
        id -> Int4,
        status_meta_id -> Nullable<Int4>,
        account_index -> Int4,
        mint -> Text,
        owner -> Nullable<Text>,
        ui_amount -> Nullable<Float8>,
        decimals -> Nullable<Int4>,
        amount -> Nullable<Text>,
        ui_amount_string -> Nullable<Text>,
    }
}

diesel::table! {
    transaction_rewards (id) {
        id -> Int4,
        status_meta_id -> Nullable<Int4>,
        pubkey -> Text,
        lamports -> Int8,
        post_balance -> Int8,
        reward_type -> Int4,
        commission -> Nullable<Int4>,
    }
}

diesel::table! {
    transaction_signatures (id) {
        id -> Int4,
        sanitized_transaction_id -> Nullable<Int4>,
        signature -> Bytea,
    }
}

diesel::table! {
    transaction_status_meta (id) {
        id -> Int4,
        transaction_id -> Nullable<Int4>,
        is_status_err -> Bool,
        error_info -> Nullable<Text>,
        fee -> Int8,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int4,
        signature -> Bytea,
        is_vote -> Bool,
        slot -> Int8,
        idx -> Int8,
    }
}

diesel::joinable!(sanitized_transactions -> transactions (transaction_id));
diesel::joinable!(transaction_inner_instruction -> transaction_inner_instructions (inner_instructions_id));
diesel::joinable!(transaction_inner_instructions -> transaction_status_meta (status_meta_id));
diesel::joinable!(transaction_log_messages -> transaction_status_meta (status_meta_id));
diesel::joinable!(transaction_post_balances -> transaction_status_meta (status_meta_id));
diesel::joinable!(transaction_post_token_balances -> transaction_status_meta (status_meta_id));
diesel::joinable!(transaction_pre_balances -> transaction_status_meta (status_meta_id));
diesel::joinable!(transaction_pre_token_balances -> transaction_status_meta (status_meta_id));
diesel::joinable!(transaction_rewards -> transaction_status_meta (status_meta_id));
diesel::joinable!(transaction_signatures -> sanitized_transactions (sanitized_transaction_id));
diesel::joinable!(transaction_status_meta -> transactions (transaction_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    sanitized_transactions,
    slots,
    transaction_inner_instruction,
    transaction_inner_instructions,
    transaction_log_messages,
    transaction_post_balances,
    transaction_post_token_balances,
    transaction_pre_balances,
    transaction_pre_token_balances,
    transaction_rewards,
    transaction_signatures,
    transaction_status_meta,
    transactions,
);
