# GFWD - GTK Firewall GUI

A modern GTK4/Libadwaita GUI application for managing firewalld zones and rules, built with Rust and Relm4.

## Overview

GFWD provides an intuitive graphical interface for configuring firewalld, the dynamic firewall daemon used in many Linux distributions. The application allows users to manage firewall zones, ports, services, and forwarding rules through a clean, modern interface developed with Libadwaita and Rust.

## Features

### Current Functionality
- **Zone Management**: View, create, and delete firewall zones
- **Port Configuration**: Add/remove ports and port forwarding rules
- **Service Control**: Start/stop firewalld service
- **Real-time Updates**: Live synchronization with firewalld state
- **Modern UI**: Native GTK4/Libadwaita interface with responsive design

### Supported Operations
- List all firewall zones
- View zone details (ports, services, rules)
- Create new zones with custom settings
- Delete existing zones
- Add/remove port rules
- Configure port forwarding
- Monitor firewalld service status
- Toggle firewalld service on/off

## Installation

### Prerequisites
- Rust toolchain (2024 edition)
- GTK4 development libraries
- Libadwaita development libraries
- firewalld installed and accessible via D-Bus

### Building from Source
```bash
# Clone the repository
git clone https://github.com/enri1196/gfwd-rs.git
cd gfwd-rs

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## Usage

### Starting the Application
```bash
./target/release/gfwd
```

### Interface Overview
- **Sidebar**: Lists all available firewall zones with status indicators
- **Main View**: Displays detailed zone configuration and settings
- **Header Bar**: Contains service controls and zone management actions

### Managing Zones
- **Create Zone**: Click the "+" button in the sidebar
- **Select Zone**: Click on any zone in the sidebar to view its configuration
- **Delete Zone**: Use the menu button or keyboard shortcut (Ctrl+D)

### Port Management
- Add ports through the zone detail view
- Configure port forwarding with destination addresses
- Remove ports and forwarding rules as needed

## Support

For issues and feature requests, please use the project's issue tracker.
