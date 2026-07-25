# Cosmic Gfwd

A libcosmic GUI for Firewalld made in Rust

## Installation

A [justfile](./justfile) is included by default for the [casey/just][just] command runner.

- `just` builds the application with the default `just build-release` recipe
- `just run` builds and runs the application
- `just install` installs the project into the system
- `just vendor` creates a vendored tarball
- `just build-vendored` compiles with vendored dependencies from that tarball
- `just check` runs clippy on the project to check for linter warnings
- `just check-json` can be used by IDEs that support LSP

## Translators

[Fluent][fluent] is used for localization of the software. Fluent's translation files are found in the [i18n directory](./i18n). New translations may copy the [English (en) localization](./i18n/en) of the project, rename `en` to the desired [ISO 639-1 language code][iso-codes], and then translations can be provided for each [message identifier][fluent-guide]. If no translation is necessary, the message may be omitted.

## Packaging

If packaging for a Linux distribution, vendor dependencies locally with the `vendor` rule, and build with the vendored sources using the `build-vendored` rule. When installing files, use the `rootdir` and `prefix` variables to change installation paths.

```sh
just vendor
just build-vendored
just rootdir=debian/cosmic-gfwd prefix=/usr install
```

It is recommended to build a source tarball with the vendored dependencies, which can typically be done by running `just vendor` on the host system before it enters the build environment.

## Permanent and runtime configuration

Zone and IP-set edits in Cosmic Gfwd modify firewalld's permanent configuration. For the selected
successfully loaded zone, the application compares every supported permanent setting with the
current runtime value. The status badge has these meanings:

- **Runtime matches permanent** means every supported value matches and firewalld returned no
  unknown setting keys.
- **Known setting differences** means the Review drawer lists exact permanent/runtime scalar
  values and order-insensitive collection membership.
- **Comparison incomplete** means known values were compared, but a newer firewalld returned
  unknown keys. Their names are shown, and synchronization is not claimed.
- **Runtime unavailable** means firewalld is inactive. Permanent zone details remain usable.

The Review drawer offers two explicit global operations:

- **Apply Permanent to Runtime** performs firewalld's state-preserving global reload. It applies all
  permanent objects and discards runtime-only changes in every zone, not only the selected zone.
- **Save Runtime as Permanent** calls `runtimeToPermanent` for the complete global runtime state,
  across all zones and firewalld objects. The result survives future reloads and restarts.

Both directions require separate destructive confirmations. Neither runs automatically. External
reloads and zone changes normally refresh the comparison through D-Bus signals; if monitoring
fails, the warning remains visible and the manual **Refresh** action remains available.

## Developers

Developers should install [rustup][rustup] and configure their editor to use [rust-analyzer][rust-analyzer]. To improve compilation times, disable LTO in the release profile, install the [mold][mold] linker, and configure [sccache][sccache] for use with Rust. The [mold][mold] linker will only improve link times if LTO is disabled.

### Application MVU architecture

Cosmic Gfwd uses a model-view-update (MVU) architecture. The root application is the sole
`cosmic::Application`: it routes feature messages, owns truly global UI state, and combines tasks
when an effect result requires work in more than one feature. Feature modules own their slice of
the model and expose module-scoped `State`, `Message`, `update`, `view`, and `effects` boundaries as
those boundaries are extracted.

Ownership is divided as follows:

| Slice | State and messages | Views | Effects |
| --- | --- | --- | --- |
| Shell/navigation | Window chrome, navigation selection, context-page visibility, menu actions, configuration, and URL launching | Header, navigation bar, context drawer shell, and application menu | Configuration subscriptions and desktop URL launching |
| Zones | Zone list/detail projections, default and active zones, firewalld status, and ordinary daemon control | Zone navigation and selected-zone details | Zone queries and mutations, status checks, and daemon start/stop |
| IP sets | IP-set list/detail projections, editor state, and mutations | IP-set navigation, details, and entry/create drawers | IP-set queries and mutations |
| Catalogs | Interface, service, and ICMP-type loading state | Catalog-backed fields rendered by dialogs | Catalog discovery and load completion |
| Reconciliation | Selected-zone permanent/runtime comparison, refresh coordination, and both global synchronization results | Reconciliation banner and review drawer | Snapshot loading, event watching, permanent-to-runtime application, and runtime-to-permanent persistence |
| Dialogs | Form input, validation, and submission intent | Feature form drawers | Submission is routed to the owning feature slice |
| Global operations | Toasts and destructive confirmation state | Toast layer and confirmation dialog | Root routing dispatches the confirmed operation to its owning slice |

`Sidebar` is transitional shell storage for zone and IP-set navigation projections. It does not
make those domains shell-owned; later slice extraction will move the projections behind their
feature boundaries without changing the visible navigation.

Sibling slices never communicate through broker-owned application state. They communicate through
root message routing and the results of effects. The broker remains UI-independent: it owns
firewalld transport and domain operations, but no application model and no libcosmic tasks.

The target root message shape is limited to `Navigation(...)`, `Zone(...)`, `IpSet(...)`,
`Catalog(...)`, `Reconciliation(...)`, `Dialog(...)`, plus root-owned toast and confirmation
messages. During incremental migration, an already-nested feature namespace may coexist with
untouched flat variants, but a migrated slice must route all of its producers, completions, and
update arms through its namespace.

### Reconciliation architecture

Selected-zone permanent/runtime reconciliation follows a one-way dependency path:

```text
UI views
    ↓
presentation model
    ↓
application reconciliation slice
    ↓
broker
    ↓
gfwd-bus proxies
```

`core/reconciliation.rs` owns typed snapshots and pure comparison semantics, while
`core/events.rs` owns pure refresh coordination and event-coalescing behavior. The
`core/broker/` module tree is the exclusive owner of system-bus connections, proxy
construction, and signal streams.

`app/reconciliation.rs` is the reconciliation model slice and owns the selected-zone lifecycle,
including request
generations, stale-response rejection, watcher health, and follow-up refresh scheduling.
`ui/reconciliation_model.rs` converts that domain state into localization-independent status,
difference groups, unknown keys, and action availability. The selected-zone banner and review
drawer both render this same presentation model.

Neither global direction is automatic. Applying permanent configuration to runtime and saving
runtime configuration permanently always remain explicit, separately confirmed operations.

[fluent]: https://projectfluent.org/
[fluent-guide]: https://projectfluent.org/fluent/guide/hello.html
[iso-codes]: https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes
[just]: https://github.com/casey/just
[rustup]: https://rustup.rs/
[rust-analyzer]: https://rust-analyzer.github.io/
[mold]: https://github.com/rui314/mold
[sccache]: https://github.com/mozilla/sccache
