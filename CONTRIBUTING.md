# Contributing

##  Contribution Workflow

To contribute a patch:

* Fork Repository 
* Create topic branch 
* Commit patches (PR, emails, ...)

In general commits should be atomic and diffs **should be easy to read**.

## Code style guide

Before writing code, please read [our code style](./CODE_STYLE.md).

## Commit format

The commit **must** be formatted in the following way:

```
<context>: <short descriptrion>

<optional description explaining better what happened in the commit>
```

If applicable, link the `issue`/`PR` to be closed with:

* Closes <url>
* Fixes <url>

The `context` name should be:

* `nostr` if changes are related to the main Rust `nostr` crate (or `protocol`?)
* `sdk`, `cli`, `pool`, `signer`, `nwc` and so on for the others Rust crates (so basically remove the `nostr-` prefix)
* `ffi(<name>)` for `UniFFI` and `js(<name>)` for `JavaScript` bindings (follow same above rules for the `<name>`)
* `book` if changes are related to the `book`
* `doc` for the `.md` files (except for `CHANGELOG.md`?)

Anything that haven't a specific context, can be left without the `<context>:` prefix (ex. change to main `justfile` commands, change to `CHANGELOG.md`?)

### Examples

```
nostr: add NIP32 support

Added kinds, tags and EventBuilder constructors to support NIP32.

Closes https://<domain>.com/rust-nostr/nostr/issue/1234
```

```
pool: fix connection issues

Long description...

Fixes https://<domain>.com/rust-nostr/nostr/issue/5612
```

```
nwc: add `pay_multiple_invoices` support

Closes https://<domain>.com/rust-nostr/nostr/issue/2222
```

```
ffi(nostr): add `EventBuilder::mute_list`
```

```
ffi(sdk): add `AbortHandle`

* Return `AbortHandle` in `Client::handle_notifications`
* Another change...
```

```
js(sdk): replace log `file path` with `module path`
```

## Deprecation policy

Where possible, breaking existing APIs should be avoided. Instead, add new APIs and
use [`#[deprecated]`](https://github.com/rust-lang/rfcs/blob/master/text/1270-deprecation.md)
to discourage use of the old one.

Deprecated APIs are typically maintained for one release cycle. In other words, an
API that has been deprecated with the 0.10 release can be expected to be removed in the
0.11 release. This allows for smoother upgrades without incurring too much technical
debt inside this library.

If you deprecated an API as part of a contribution, we encourage you to "own" that API
and send a follow-up to remove it as part of the next release cycle.

## Unwrap and expect

Usage of `.unwrap()` or `.expect("...")` methods is allowed **only** in `examples` or `tests`.

## Coding Conventions

Use `just precommit` or `just check` to format and check the code before committing. This is also enforced by the CI.
