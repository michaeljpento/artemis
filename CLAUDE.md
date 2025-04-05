# Artemis Development Guidelines

## Commands
- Build: `cargo build`
- Test all: `cargo test --all`
- Test single: `cargo test -p <crate-name> <test_name>` (e.g., `cargo test -p artemis-core test_block_collector_sends_blocks`)
- Lint: `cargo clippy --all --all-features`
- Format: `cargo +nightly fmt --all`
- Contract tests: `forge test --root ./contracts`
- Generate bindings: `just generate-bindings`

## Code Style
- Follow Rust idioms: `#![deny(unused_must_use, rust_2018_idioms)]`
- Module organization: collectors, strategies, executors, types
- Document public interfaces with doc comments (//!)
- Use descriptive variable names with snake_case
- Error handling: Use Result<T, E> with proper propagation
- Tests: Use #[tokio::test] for async tests, place in crate's /tests directory
- Imports: Group standard library, external crates, then internal modules
- Avoid unused dependencies: `#![warn(unused_crate_dependencies)]`
- Types: Prefer strong typing, use Arc<T> for shared ownership
- Format with nightly rustfmt
- Use tokio for async runtime

Artemis is a modular MEV bot framework following event-driven architecture with Collectors, Strategies, and Executors.