use zbus_macros::proxy;

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    gen_blocking = false,
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
/// Proxy for the `org.freedesktop.NetworkManager` interface
pub trait NetworkManager {
    /// Get the list of realized network devices
    #[zbus(name = "GetDevices")]
    fn get_devices(&self) -> zbus::Result<Vec<zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    gen_blocking = false,
    default_service = "org.freedesktop.NetworkManager"
)]
/// Proxy for the `org.freedesktop.NetworkManager.Device` interface
pub trait Device {
    /// The name of the device's control (and often data) interface
    #[zbus(property)]
    fn interface(&self) -> zbus::Result<String>;

    /// The device type
    #[zbus(property)]
    fn device_type(&self) -> zbus::Result<u32>;
}
