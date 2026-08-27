---
status: accepted
---

# API keys live in the settings file, not the OS keychain

Keys are stored in `settings.toml` (mode `0600` on Unix) with environment
variables taking precedence, rather than in the OS keychain. The keychain is the
more private option, but on macOS the right to read a keychain item is bound to
the code signature, and an unsigned build's ad-hoc signature changes on every
rebuild — so until the app carries a Developer ID (see ADR-0004) the keychain
means a permission dialog on every launch, and one after every rebuild during
development. For a utility whose entire premise is removing friction, that trade
goes the other way.

Resolution order, mirroring the sibling project `plz`: a Provider's `api_key_env`
field, then the preset's conventional variable (`DEEPSEEK_API_KEY`,
`OPENAI_API_KEY`, `OPENROUTER_API_KEY`), then the `api_key` field in the file.

## Consequences

The key is readable by any process running as the same user. This is a conscious
trade, not an oversight. To keep the reversal cheap, all key access goes through
a single interface in the Rust layer, so moving to the keychain later is a change
to one module rather than a change everywhere; the natural moment to reconsider
is when the app gets signed.

The key never enters the webview, which renders untrusted model output.
