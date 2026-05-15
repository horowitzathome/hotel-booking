---
name: security-review
description: >-
  Security review checklists, threat model, severity classification,
  and dependency verification for Rust applications.
  Load when conducting security reviews.
compatibility:
  - claude-code
  - opencode
  - github-copilot
metadata:
  version: "1.0"
  author: team
---

## Core Security Principles

### Security as Emergent Property

Security cannot be bolted on later. Verify that security considerations are present from initial design, not added as an afterthought.

### Defense in Depth

Multiple overlapping controls. No single mechanism should be the only protection. Check for:
- Input validation at entry points AND internal processing
- TLS for transport AND credential protection at rest
- Timeout enforcement at multiple layers

### Least Privilege

Grant minimal necessary permissions:
- Code accesses only required resources
- Credentials scoped to specific operations
- No unnecessary capabilities in container/service

### Fail Secure

When errors occur, the system should remain secure:
- Connection failures should not expose credentials
- Parsing errors should not bypass validation
- Resource exhaustion should not disable security checks

## Security Checklist

### Memory Safety (Rust-Specific)

- [ ] No `unsafe` blocks without `// SAFETY:` comment proving the invariant holds
- [ ] `unsafe` blocks are as small as possible — extract to a dedicated `unsafe fn`
- [ ] `unwrap()` absent in all non-test code paths (signals a panic vector)
- [ ] Integer arithmetic uses checked/saturating/wrapping variants where overflow is possible
- [ ] No raw pointer arithmetic without exhaustive bounds checking
- [ ] `transmute` absent — use safe conversion APIs instead
- [ ] No use-after-free risk from raw pointer aliasing

### Path Traversal and File Operations

- [ ] Input paths resolved to absolute paths before use (`canonicalize()`)
- [ ] No directory traversal via crafted input (`../`, symlinks)
- [ ] File operations restricted to configured directories
- [ ] Symlinks not followed unless explicitly required (use `metadata()` not `symlink_metadata()`)
- [ ] Output files written only to expected locations
- [ ] Agent/development temp files only in `.scratch/tmp/`, never system `/tmp`

### Input Injection

- [ ] User-derived content escaped before inclusion in output (HTML, JSON, SQL)
- [ ] No inputs passed to shell commands (`std::process::Command` with user data)
- [ ] No string interpolation for SQL — use parameterized queries (`sqlx`, `diesel`)
- [ ] Regex patterns bounded with `regex::RegexBuilder::size_limit()` (prevent ReDoS)
- [ ] Log injection prevented — newlines stripped/escaped in log field values

### Serialization Safety (Serde)

- [ ] `serde` used with `deny_unknown_fields` where strict parsing is needed
- [ ] Deserialized data treated as untrusted until validated
- [ ] No arbitrary type deserialization (`serde_json::Value` used only at parse boundaries)
- [ ] Corrupted data files handled gracefully (`serde_json::from_str` returns `Result`)
- [ ] No recursive types that could cause stack overflow during deserialization

### HTML Output Safety (if applicable)

- [ ] All user-derived content HTML-escaped (`html_escape` crate or equivalent)
- [ ] `<`, `>`, `&`, `"`, `'` escaped in text content and attributes
- [ ] No inline JavaScript in generated HTML
- [ ] No external resource loading with remote URLs in generated markup
- [ ] `href` attributes use relative paths only

### Credential and Sensitive Data Handling

- [ ] Tokens never logged (even at DEBUG level)
- [ ] Credentials not hardcoded — loaded from environment or config file
- [ ] Sensitive data zeroed on drop (`zeroize` crate for secrets in memory)
- [ ] No credentials in URLs or query strings (use headers or request body)
- [ ] Secrets not included in `Debug` output (use `#[debug = "..."]` or manual `Debug` impl)
- [ ] `secrecy::Secret<T>` used for sensitive values in long-lived structs

### Input Validation

- [ ] All user-facing inputs validated at system boundaries (CLI args, HTTP body, file content)
- [ ] Numeric inputs checked for overflow before casting
- [ ] String lengths bounded (no unbounded allocation from user data)
- [ ] Configuration values validated at startup with descriptive errors

### Network Security (if applicable)

- [ ] Connection and read timeouts set on all HTTP operations (`reqwest` timeout config)
- [ ] No hardcoded URLs — loaded from configuration
- [ ] TLS validation enabled (no `danger_accept_invalid_certs`)
- [ ] Responses from external services treated as untrusted — validate before use

### Resource Management

- [ ] No unbounded memory allocation from user-controlled sizes
- [ ] File handles closed via `Drop` (RAII — Rust guarantees this; verify no `mem::forget`)
- [ ] Stream operations do not hold references to large buffers after processing
- [ ] Graceful behavior under high load — bounded channels, backpressure

### Dependency Security

- [ ] `cargo audit` passes with no known CVEs
- [ ] No unnecessary dependencies in `Cargo.toml`
- [ ] `serde_json`, `reqwest`, `tokio` versions checked for known issues
- [ ] Dependencies from crates.io only (no `git` deps in production unless justified)
- [ ] `Cargo.lock` committed for applications (not for libraries)

### Logging Safety

- [ ] No sensitive data in log output
- [ ] `tracing` structured fields used (not string concatenation)
- [ ] No `println!` or `eprintln!` in production code
- [ ] Log messages include sufficient context for debugging

## Severity Classification

### CRITICAL (BLOCKED)

- Credential exposure in logs or error messages
- Remote code execution via `unsafe` misuse or command injection
- Authentication bypass
- Unvalidated external input to `unsafe` operations
- `unwrap()` on untrusted data that causes panic in a server context

### HIGH (BLOCKED)

- Path traversal allowing writes outside designated directories
- Missing input validation on external data
- Unbounded memory allocation from user-controlled sizes
- Integer overflow in size calculations
- ReDoS-vulnerable regex patterns

### MEDIUM

- Sensitive data in verbose error messages
- Missing timeouts on network operations
- TLS certificate validation disabled
- Missing `zeroize` on long-lived secrets

### LOW

- Information disclosure in error messages (non-sensitive)
- Missing rate limiting
- Dependency not on latest patch version

## Detection Patterns

Use Grep to search for dangerous code patterns during review:

| Pattern | Where to Search | What It Detects |
|---|---|---|
| `unwrap()` | `src/` (excluding `#[cfg(test)]` blocks) | Panic vector on untrusted data |
| `unsafe` | `src/` | Unsafe blocks requiring safety proof |
| `transmute` | `src/` | Unsafe type reinterpretation |
| `Command::new\|std::process::Command` | `src/` | Shell execution with potential injection |
| `format!\|write!\|println!` with user data | `src/` | Unescaped output |
| `/tmp/` | `src/` | System tmp usage (should use `.scratch/tmp/`) |
| `danger_accept_invalid` | `src/` | TLS validation disabled |
| `mem::forget` | `src/` | RAII bypass (resource leak or use-after-free) |
