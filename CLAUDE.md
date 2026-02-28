# Rust Project — Claude Instructions

## Mandatory Rules

1. **Always write tests** alongside production code — no feature ships without tests
2. **Always verify tests pass** after every change — the PostToolUse hook runs automatically;
   if it shows failures, fix them before moving on
3. Run `cargo clippy -- -D warnings` and resolve all warnings
4. Run `cargo fmt` before considering any task complete

## Available MCP Tools

Install with `curl -sSf https://raw.githubusercontent.com/USUARIO/claude-rust-scaffold/main/install.sh | sh`

| Server | Tools | Purpose |
|--------|-------|---------|
| `rust-mcp` | `cargo_check`, `cargo_build`, `cargo_test`, `cargo_clippy`, `cargo_fmt`, `cargo_add` | Run cargo commands directly |
| `crates` | search, versions, dependencies, docs | Explore crates.io and docs.rs |

## Test Structure

### Unit Tests — inside `src/`

Place at the bottom of each source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_error_when_input_is_empty() {
        // arrange
        let input = "";
        // act
        let result = parse(input);
        // assert
        assert!(result.is_err());
    }
}
```

- Name tests descriptively: `should_<outcome>_when_<condition>`
- Cover: happy path, edge cases (empty, max values), error cases

### Integration Tests — `tests/` directory

- One file per feature or behavior
- Use only public interfaces (`pub`)
- Simulate real usage end-to-end

```rust
// tests/parsing.rs
use tmuxido::parse;

#[test]
fn parses_valid_input_successfully() {
    let result = parse("valid input");
    assert!(result.is_ok());
}
```

### Snapshot Testing with `insta`

For complex outputs or large structs:

```rust
#[test]
fn renders_report_correctly() {
    let report = generate_report(&data);
    insta::assert_snapshot!(report);
}
```

Review snapshots: `cargo insta review`

### Property Testing with `proptest`

For pure functions over wide input domains:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn round_trip_encode_decode(s in ".*") {
        let encoded = encode(&s);
        prop_assert_eq!(decode(&encoded), s);
    }
}
```

## Recommended `Cargo.toml` dev-dependencies

```toml
[dev-dependencies]
proptest = "1"
insta = { version = "1", features = ["json", "yaml"] }
mockall = "0.13"

# if async:
tokio = { version = "1", features = ["full", "test-util"] }
```

## Recommended Project Structure

```
tmuxido/
├── Cargo.toml
├── src/
│   ├── lib.rs          # core logic (unit tests at bottom)
│   ├── main.rs         # entrypoint (thin, delegates to lib)
│   └── module/
│       └── mod.rs      # #[cfg(test)] mod tests {} at bottom
├── tests/
│   └── integration.rs  # integration tests
└── benches/
    └── bench.rs        # benchmarks (optional)
```

Prefer `lib.rs` + `main.rs` split so logic stays testable independently of the binary entrypoint.
