# pb-mapper maintenance

This fork publishes `portmapper-plus` with the Rust library name `portmapper`.
The baseline is upstream `portmapper 0.20.0`, commit
`80b449c74d38c564c598ae286de13032276f3515` in `n0-computer/net-tools`.
`dev` carries changes; `upstream` retains the unmodified baseline. Transitive
`netwatch` comes from crates.io.

The optional `Observer` reports actual discovery, protocol selection, mapping,
renewal (including an unchanged external address), expiry and cleanup events.
It does not change protocol selection or introduce a polling task. Callbacks
must be non-blocking and must not call back into the service.
`deactivate_and_wait()` acknowledges cleanup before an owner closes its event
stream. PCP/NAT-PMP release means a deletion packet was sent; only successful
UPnP cleanup confirms the gateway response.

Validate with `cargo fmt --all -- --check`,
`cargo clippy -p portmapper-plus --all-targets --all-features -- -D warnings`
and `cargo test -p portmapper-plus --all-features`. The observer test uses a
loopback PCP gateway for success, unchanged-address renewal, rejection and
release; it does not modify a host router.

Publish this package before dependent pb-mapper SDK crates. Never reuse a
published version for changed source.
