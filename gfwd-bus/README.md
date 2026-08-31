# gfwd-bus

`gfwd-bus` provides typed, asynchronous Rust proxies for the firewalld D-Bus
API, plus small proxies for the system services used alongside firewalld.
It is the D-Bus boundary used by [GFWD](https://github.com/enri1196/gfwd-rs).

The crate does not manage a D-Bus connection or impose application state
management. It describes remote interfaces with [`zbus`](https://docs.rs/zbus)
and lets the caller decide how connections, errors, and application logic are
owned.

## Architecture

```text
Application (for example, cosmic-gfwd)
                    |
                    v
          zbus::Connection::system()
                    |
                    v
          generated *Proxy types
                    |
                    v
             System D-Bus
          /          |           \
         v           v            v
     firewalld     systemd    NetworkManager
```

The firewalld interfaces are grouped by their D-Bus role:

- Runtime interfaces: `firewalld1`, `zone`, `ipset`, `policies`, and `direct`.
- Permanent configuration interfaces: `config_firewalld1`, `config_zone`,
  `config_ipset`, `config_service`, `config_icmptype`, `config_helpers`,
  `config_policies`, and `config_direct`.
- Related system services: `systemd` and `network_manager`.

Each module is gated by a feature with the same name, and there are no default
features. Enable only the interfaces an application uses; docs.rs builds with
all features so the complete API is visible there.

## Example

```rust,no_run
use gfwd_bus::firewalld1::FirewallD1Proxy;
use zbus::Connection;

async fn show_default_zone() -> zbus::Result<()> {
    let connection = Connection::system().await?;
    let firewalld = FirewallD1Proxy::new(&connection).await?;

    println!("default zone: {}", firewalld.get_default_zone().await?);
    Ok(())
}
```

Enable the corresponding feature in `Cargo.toml`:

```toml
[dependencies]
gfwd-bus = { version = "0.1.1", features = ["firewalld1"] }
```

The generated proxy types expose the async methods, properties, and signal
helpers defined by the corresponding D-Bus interfaces. Refer to the API docs
for the exact method signatures and wire names.
