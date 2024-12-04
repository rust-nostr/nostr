# Contributing to rust-nostr

## Contribution Workflow

Before writing code, please read [our code style](./CODE_STYLE.md).

To contribute a patch:

* Fork Repository 
* Create topic branch 
* Commit patches (PR, emails, ...)

In general commits should be atomic and diffs **easy to read**.

## Commit Style

The commit **must** be formatted as following:

```
<context>: <short descriptrion>

<optional description explaining reasons for the changes>
```

If applicable, link the `issue`/`PR` to be closed with:

* Closes <url>
* Fixes <url>

The `context` should be:

* `nostr` if changes are related to the main Rust `nostr` crate (or `protocol`?)
* `sdk`, `cli`, `pool`, `signer`, `nwc` and so on for the others Rust crates (so remove the `nostr-` prefix)
* `ffi` for `UniFFI`, `js` for `JavaScript` bindings and so on
* `book` if changes are related to the `book`
* `doc` for the `.md` files (except for `CHANGELOG.md`?)

Anything that does not have a specific context can be left without the `<context>:` prefix,
e.g. a change to the main `justfile` commands or change to `CHANGELOG.md`.

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
ffi: expose `EventBuilder::mute_list`
```

```
ffi: add `AbortHandle`

* Return `AbortHandle` in `Client::handle_notifications`
* Another change...
```

```
js: replace log `file path` with `module path`
```

## Deprecation policy

Where possible, breaking existing APIs should be avoided.
Instead, add new APIs and use 
[`#[deprecated]`](https://github.com/rust-lang/rfcs/blob/master/text/1270-deprecation.md)
to discourage use of the old one.

Deprecated APIs are typically maintained for one release cycle.
In other words, an API that has been deprecated with the 0.10 release 
can be expected to be removed in the 0.11 release.
This allows for smoother upgrades without incurring too much technical debt inside this library.

If you deprecated an API as part of a contribution, we encourage you to "own" that API
and send a follow-up to remove it as part of the next release cycle.

## Coding Conventions

Usage of `.unwrap()` or `.expect("...")` methods is allowed **only** in `examples` or `tests`.

Install https://github.com/casey/just and use `just precommit` or `just check` 
to format and check the code before committing.
This is also enforced by the CI.
