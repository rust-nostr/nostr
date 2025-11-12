# Reporting a Vulnerability

1. **Preferred channel:** Open a private report via GitHub Security Advisories: <https://github.com/rust-nostr/nostr/security/advisories/new>. This keeps the discussion confidential until a fix ships.
2. **Alternative channel:** If GitHub is unavailable for you, send an encrypted message following the instructions in the organization guidelines (<https://github.com/rust-nostr/guidelines>). That document lists the current security PGP keys.
3. **What to include:** affected crate(s) and versions, a minimal proof-of-concept, the impact you observed, and any suggested mitigation ideas. Logs and environment info (`rustc -V`, OS, enabled features) are extremely helpful.
4. **Response targets:** we aim to acknowledge new reports within **5 business days** and keep you posted every time we cross a milestone (triage, fix ready, release, disclosure).

Please do **not** open public issues for security problems. We appreciate coordinated disclosure and will credit you in the release notes unless you ask otherwise.
