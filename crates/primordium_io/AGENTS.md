# Agent Project Memory: Primordium I/O

> I/O and persistence layer for state serialization, P2P communication, and blockchain anchoring.

---

## WHERE TO LOOK

| Task | File |
|------|------|
| Serialization | `serialization.rs` (JSON, HexDNA), `persistence.rs` (rkyv) |
| Network | `network/quic.rs` (QUIC + Ed25519), `network/flow_control.rs` (TokenBucket) |
| Errors | `error.rs` (thiserror-based IoError) |
| Registry | `registry.rs` (LineagePersistence trait) |
| History | `history.rs` (event streaming, fossil records) |

---

## CONVENTIONS

### Serialization

- **JSON**: Human-readable config/registry. Use `to_json_pretty()` for files.
- **HexDNA**: Base16-encoded JSON for portable genotype export/import.
- **rkyv**: Zero-copy snapshots. Requires `Archive` derive and `AlignedVec` for alignment.

### Network Security

- **Ed25519**: Sign all `AuthorityTransfer` messages with `sign()`, verify with `verify()`.
- **Anti-replay**: Include `nonce` and `timestamp` in signed payloads.
- **Self-signed certs**: QUIC uses `SkipServerVerification` (trust-on-first-use).

### Error Handling

- Use `IoError` with context: `IoError::FileSystem(e).with_context("reading config")`
- Prefer constructors: `IoError::validation("empty hex")` over direct variants.
- All I/O functions return `Result<T>`.

### Rate Limiting

- `TokenBucket` for throttling. Initialize with capacity and refill rate (tokens/sec).
- Thread-safe via `Arc<Mutex<>>`. Call `try_acquire()` before sending.

---

## ANTI-PATTERNS

### ❌ Don't skip validation

```rust
// BAD
let bytes = hex::decode(hex_str)?;
let json = String::from_utf8(bytes)?;
let data: T = serde_json::from_str(&json)?;
// GOOD
let bytes = hex::decode(hex_str)
    .map_err(|e| IoError::validation(format!("Invalid hex: {}", e)))?;
if bytes.is_empty() {
    return Err(IoError::validation("Empty hex data"));
}
```

### ❌ Don't forget atomic writes

```rust
// BAD - Corrupts on crash
let file = File::create(path)?;
serde_json::to_writer_pretty(file, data)?;
// GOOD - Atomic via temp file
let tmp_path = path.with_extension("tmp");
{
    let file = File::create(&tmp_path)?;
    serde_json::to_writer_pretty(file, data)?;
}
std::fs::rename(tmp_path, path)?;
```

### ❌ Don't ignore rkyv alignment

```rust
// BAD - May crash
let archived = unsafe { rkyv::archived_root::<T>(&bytes) };
// GOOD
let mut aligned = AlignedVec::with_capacity(bytes.len());
aligned.extend_from_slice(&bytes);
let archived = rkyv::check_archived_root::<T>(&aligned)?;
```
