# Heimdall Codebase Architecture

This document provides a high-level overview of the `heimdall` project structure, its core components, and the data flow through the system.

## 1. Top-Level Directory Structure

The project root is organized as a monorepo containing the core Rust services and configuration files.

-   `config/`: Contains configuration files, primarily for the Geyser plugin.
-   `packages/`: A Rust workspace containing all the individual services (crates). This is the heart of the application.
-   `Cargo.toml`: Defines the root Rust workspace, declaring all sub-directories in `packages/` as members.
-   `docker-compose.yml`: Defines the local development environment, spinning up required infrastructure like PostgreSQL and Redis.
-   `Dockerfile`: Used to build a single, optimized production container image containing all the compiled service binaries.
-   `fly.toml`: (Example) A configuration file for deploying the services to a platform like [Fly.io](https://fly.io), defining processes and public-facing services.

## 2. Core Services (in `packages/`)

The application is broken down into several independent but interconnected services.

### `laser-ingest/` (Recommended Ingest Method)

* **Role:** Helius LaserStream Ingest Worker
* **Architecture (v2 – modular processors)**
    1. **Watch-list** – A JSON file in dev (`watchlist.json`) or a live Supabase table in prod supplies a list of pools to stream:   
       `{ pool_id, variant = [dbc|amm|damm], quote_vault?, ... }`.
    2. **Processor registry** – Each pool variant registers a `Processor` (see `packages/laser-ingest/src/processors/`).  
       • `DbcProcessor` – parses Dynamic Bonding Curve swaps & quote-vault balances.  
       • `AmmProcessor`, `DammProcessor` – stubs ready for AMM / DAMM logic.  
    3. **Filter builder** – For every pool the processor contributes the exact Helius `SubscribeRequest` filters it needs (accounts, transactions). These are merged into one connection.
    4. **Dispatcher** – Every incoming `SubscribeUpdate` is matched to a pool and routed to the correct processor.  
       Processors publish **one uniform Redis stream** `heimdall:pools:swaps` containing JSON `{ pool_id, variant, ... }`.
* **Output:**
    • `heimdall:pools:swaps` – variant-agnostic swap events.  
    • Additional streams can be added per processor (e.g. quote-vault balances) but core consumers rely on the uniform stream.
* **Why:** This design lets us add new pool types or individual pools **without redeploying** – insert a row in the watch-list table and the worker reconnects with new filters.

### `db-processor/`

-   **Role:** Persistence Worker
-   **Function:** This is a background service that runs continuously. It listens to the Redis Streams populated by the ingest worker (`laser-ingest` or `geyser`