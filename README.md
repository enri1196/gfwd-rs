# GFWD - COSMIC Firewall Configuration

A COSMIC-native application for managing firewalld zones and rules, built with Rust and libcosmic.

## Overview

GFWD provides a graphical interface for configuring firewalld, the dynamic firewall daemon used by many Linux distributions. The application manages permanent zone configuration and offers an explicit action for applying those changes to firewalld's runtime state.

## Features

### Current Functionality

- **Zone Management**: View, create, delete, and select the default firewall zone
- **Port Configuration**: Add/remove ports and port forwarding rules
- **Zone Rules**: Manage services, interfaces, sources, ICMP blocks, rich rules, and IP sets
- **Service Control**: View and start/stop `firewalld.service`
- **Validation and Feedback**: Validate structured input and report asynchronous operation results
- **Modern UI**: Native libcosmic interface with context drawers, dialogs, and toasts

### Supported Operations

- List all firewall zones
- View zone details and active-zone badges
- Create and delete zones
- Add and remove ports and forwarding rules
- Toggle masquerading and ICMP block inversion
- Manage services, interfaces, sources, and ICMP blocks
- Build structured rich rules or enter advanced raw rules
- Create and delete IP sets and manage type-aware entries
- Monitor and start or stop firewalld
- Explicitly reload permanent changes into runtime configuration

## Installation

### Prerequisites

- Rust toolchain (2024 edition)
- libcosmic build dependencies
- firewalld installed and accessible through D-Bus
- systemd for service status and start/stop controls

### Building from Source

```bash
git clone https://github.com/enri1196/gfwd-rs.git
cd gfwd-rs
cargo build --release --package cosmic-gfwd
```

## Usage

```bash
./target/release/cosmic-gfwd
```

- **Sidebar**: Lists firewall zones, their active interfaces, sources, and default-zone status
- **Main View**: Displays the selected zone's permanent configuration
- **Header Bar**: Contains service controls, runtime apply, and zone management actions

See [cosmic-gfwd/README.md](cosmic-gfwd/README.md) for configuration semantics and migration scope.

## Support

For issues and feature requests, please use the project's issue tracker.
