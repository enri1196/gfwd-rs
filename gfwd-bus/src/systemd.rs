use zbus::{Result as ZResult, zvariant::OwnedObjectPath};
use zbus_macros::proxy;

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    gen_blocking = false,
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
pub trait Manager {
    #[zbus(name = "StartUnit")]
    fn start_unit(&self, name: &str, mode: &str) -> ZResult<OwnedObjectPath>;

    #[zbus(name = "StopUnit")]
    fn stop_unit(&self, name: &str, mode: &str) -> ZResult<OwnedObjectPath>;

    #[zbus(name = "GetUnit")]
    fn get_unit(&self, name: &str) -> ZResult<OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.systemd1.Unit",
    gen_blocking = false,
    default_service = "org.freedesktop.systemd1"
)]
pub trait Unit {
    #[zbus(property)]
    fn active_state(&self) -> ZResult<String>;
}
