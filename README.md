# heimdall

solana geyser plugin with redis, grpc, and postgres for real-time blockchain data streaming, analytics, and client updates
<img width="919" height="671" alt="Screenshot 2025-07-18 at 7 25 20 AM" src="https://github.com/user-attachments/assets/5e042e4b-425a-4e13-b849-d4c3e79d682c" />

## installation

### building from source

#### prerequisites

- rust (latest stable version)
- docker and docker compose
- diesel cli for database migrations

#### build

```shell
cargo build --release
```

builds:

- linux: `./target/release/libgeyser.so`
- macos: `./target/release/libgeyser.dylib`

**important:** solana's plugin interface requires the build environment of the solana validator and this plugin to be **identical**.

this includes the solana version and rust compiler version.
loading a plugin targeting wrong versions will result in memory corruption and crashes.

## configuration

configuration is specified via json config file.

### example config

```json
{
  "libpath": "target/release/libgeyser.dylib",
  "redis": {
    "url": "redis://localhost:6379",
    "max_connections": 10,
    "connection_timeout_ms": 5000,
    "database": 0
  },
  "shutdown_timeout_ms": 30000,
  "filters": [{
    "update_account_stream": "heimdall:accounts",
    "slot_status_stream": "heimdall:slots", 
    "transaction_stream": "heimdall:transactions",
    "program_ignores": [
      "Sysvar1111111111111111111111111111111111111",
      "Vote111111111111111111111111111111111111111"
    ],
    "program_filters": [],
    "account_filters": [],
    "publish_all_accounts": false,
    "include_vote_transactions": true,
    "include_failed_transactions": true,
    "wrap_messages": false
  }],
  "prometheus": "127.0.0.1:9090"
}
```

### reference

- `libpath`: path to geyser plugin
- `redis`: redis connection configuration
  - `url`: redis connection url
  - `max_connections`: maximum number of connections in pool
  - `connection_timeout_ms`: connection timeout in milliseconds
  - `database`: redis database number
- `shutdown_timeout_ms`: time the plugin is given to flush out all messages upon exit
- `prometheus`: optional address to provide metrics in prometheus format
- `filters`: array of filters with the following fields:
  - `update_account_stream`: redis stream name for account updates
  - `slot_status_stream`: redis stream name for slot status updates
  - `transaction_stream`: redis stream name for transaction updates
  - `program_ignores`: account addresses to ignore
  - `program_filters`: solana program ids to include
  - `account_filters`: solana accounts to include
  - `publish_all_accounts`: publish all accounts on startup
  - `include_vote_transactions`: include vote transactions
  - `include_failed_transactions`: include failed transactions
  - `wrap_messages`: wrap all messages in unified wrapper object

## setup

### infrastructure setup

1. start docker services

```shell
docker compose up -d
```

this starts postgresql and redis containers with the following default configuration:

- postgresql: `postgresql://heimdall:heimdall@localhost:5432/heimdall_db`
- redis: `redis://localhost:6379`

2. install diesel cli

```shell
cargo install diesel_cli --no-default-features --features postgres
```

3. configure environment variables

```shell
export DATABASE_URL="postgresql://heimdall:heimdall@localhost:5432/heimdall_db"
```

4. run database migrations

```shell
pnpm run migrate
```

### running services

start services in separate terminals:

1. start api server

```shell
pnpm run start:api
```

2. start stream server

```shell
pnpm run start:stream
```

3. start database processor

```shell
pnpm run start:db-processor
```

4. test stream client

```shell
pnpm run start:client
```

## development

available scripts:

- `pnpm run build` - build in release mode
- `pnpm run build:dev` - build in debug mode
- `pnpm run build:geyser` - build geyser plugin specifically
- `pnpm run test` - run tests
- `pnpm run check` - check code without building
- `pnpm run clean` - clean build artifacts
- `pnpm run migrate` - run database migrations
- `pnpm run migrate:reset` - reset and rerun migrations

## buffering

the redis publisher acts strictly non-blocking to allow the solana validator to sync without induced lag.
this means incoming events from the solana validator get buffered and published asynchronously.

when the publishing buffer is exhausted, additional events will get dropped.
this can happen when redis is slow or the connection fails.
therefore it is crucial to choose sufficiently large buffer sizes.

buffer size can be controlled using redis configuration options and connection pool settings.

## license

licensed under the apache license, version 2.0
