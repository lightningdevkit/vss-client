# 0.6.0 - Jun 1, 2026

> [!WARNING]
> This release of `vss-client-ng` requires a corresponding upgrade to `vss-server`. `vss-server`
> should be upgraded first to at least `0.1.0-alpha.1`, then `vss-client-ng` should be upgraded
> to this release.

## Features and API updates
- Assert the VSS protocol version matches (#61).

In total, this release features 7 files changed, 93 insertions, 29 deletions from 1 author:
- Leo Nash

# 0.5.0 - Feb 20, 2026

## Features and API updates
- Add a module to authenticate by proving private key knowledge (#54).
- Switch from `reqwest` to `bitreq` HTTP client (#56).

## Bug Fixes and Improvements
- Switch to `bitreq::Url` (#58).

In total, this release features 12 files changed, 218 insertions, 140 deletions from 3 authors in alphabetical order:
- Elias Rohrer
- Leo Nash
- Matt Corallo

# 0.4.1 - Jan 9, 2026

## Features and API updates
- Add `trace`-level logging via the `log` facade (#51).

In total, this release features 7 files changed, 99 insertions, 9 deletions from 3 authors in alphabetical order:
- Andrei
- Elias Rohrer
- tankyleo

# 0.4.0 - Nov 11, 2025

## Crate Metadata
- Rename `vss-client` crate to `vss-client-ng` (#46).
- Move repository from <https://github.com/lightningdevkit/rust-vss-client> to <https://github.com/lightningdevkit/vss-client> (#48).

## Features and API updates
- Bump MSRV to 1.75.0 (#38).
- Add `VssClient::from_client_and_headers` constructor (#39).
- Allow to set `aad` in `StorableBuilder::{build, deconstruct}` (#40).
- Pass the `data_encryption_key` by reference to `StorableBuilder::{build, deconstruct}` (#40).

## Bug Fixes and Improvements
- Set a 10 second timeout on all HTTP client requests (#39).

In total, this release features 15 files changed, 205 insertions, 1162 deletions in 19 commits from 2 authors in alphabetical order:
- Elias Rohrer
- Leo Nash
