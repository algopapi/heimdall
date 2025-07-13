// @generated automatically by Diesel CLI.

diesel::table! {
    orders (id) {
        id -> Int8,
        user_id -> Int4,
        pool_id -> Int4,
        protocol_id -> Int4,
        #[max_length = 20]
        order_type -> Varchar,
        #[max_length = 4]
        side -> Varchar,
        price -> Nullable<Numeric>,
        amount -> Numeric,
        filled_amount -> Numeric,
        #[max_length = 20]
        status -> Varchar,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    pools (id) {
        id -> Int4,
        protocol_id -> Int4,
        #[max_length = 44]
        pool_pubkey -> Varchar,
        #[max_length = 44]
        base_mint -> Varchar,
        #[max_length = 44]
        quote_mint -> Varchar,
        base_decimals -> Int2,
        quote_decimals -> Int2,
        fee_numerator -> Int8,
        fee_denominator -> Int8,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    protocols (id) {
        id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 44]
        program_id -> Varchar,
        description -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int8,
        pool_id -> Int4,
        protocol_id -> Int4,
        user_id -> Nullable<Int4>,
        #[max_length = 88]
        tx_signature -> Varchar,
        #[max_length = 20]
        tx_type -> Varchar,
        amount_in -> Numeric,
        amount_out -> Numeric,
        #[max_length = 44]
        token_in -> Varchar,
        #[max_length = 44]
        token_out -> Varchar,
        price -> Nullable<Numeric>,
        fee -> Numeric,
        slot -> Int8,
        block_time -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_swaps (id) {
        id -> Int8,
        user_id -> Int4,
        tx_id -> Int8,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 44]
        pubkey -> Varchar,
        signature -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(orders -> pools (pool_id));
diesel::joinable!(orders -> protocols (protocol_id));
diesel::joinable!(orders -> users (user_id));
diesel::joinable!(pools -> protocols (protocol_id));
diesel::joinable!(transactions -> pools (pool_id));
diesel::joinable!(transactions -> protocols (protocol_id));
diesel::joinable!(transactions -> users (user_id));
diesel::joinable!(user_swaps -> transactions (tx_id));
diesel::joinable!(user_swaps -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    orders,
    pools,
    protocols,
    transactions,
    user_swaps,
    users,
);
