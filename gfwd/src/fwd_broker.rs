use std::io::Result;

use tokio::sync::OnceCell;

pub struct FwdBroker {
    pub fwd: gfwd_bus::firewalld1::FirewallD1Proxy<'static>,
    pub cfg_fwd: gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy<'static>,
}

// Static object holding the sender side of the channel
static BROKER: OnceCell<FwdBroker> = OnceCell::const_new();

impl FwdBroker {
    pub async fn get_broker() -> &'static FwdBroker {
        BROKER.get_or_init(|| async move {
            FwdBroker {
                fwd: gfwd_bus::firewalld1::new_firewalld_proxy().await.unwrap(),
                cfg_fwd: gfwd_bus::config_firewalld1::new_config_firewalld1_proxy().await.unwrap(),
            }
        }).await
    }

    pub async fn get_zones(&self) -> Result<Vec<String>> {
        self.cfg_fwd.get_zone_names().await
            .map_err(|e| std::io::Error::new(get_kind(&e), e))
    }
}

fn get_kind(zbus_error: &zbus::Error) -> std::io::ErrorKind {
    match zbus_error {
        zbus::Error::InterfaceNotFound => std::io::ErrorKind::NotFound,
        zbus::Error::Address(_) => std::io::ErrorKind::AddrNotAvailable,
        zbus::Error::InputOutput(_) => std::io::ErrorKind::Other,
        zbus::Error::InvalidField => std::io::ErrorKind::InvalidData,
        zbus::Error::ExcessData => std::io::ErrorKind::Other,
        zbus::Error::Variant(_) => std::io::ErrorKind::Other,
        zbus::Error::Names(_) => std::io::ErrorKind::Other,
        zbus::Error::IncorrectEndian => std::io::ErrorKind::Other,
        zbus::Error::Handshake(_) => std::io::ErrorKind::Other,
        zbus::Error::InvalidReply => std::io::ErrorKind::Other,
        zbus::Error::MethodError(_, _, _) => std::io::ErrorKind::Other,
        zbus::Error::MissingField => std::io::ErrorKind::Other,
        zbus::Error::InvalidGUID => std::io::ErrorKind::Other,
        zbus::Error::Unsupported => std::io::ErrorKind::Other,
        zbus::Error::FDO(_) => std::io::ErrorKind::Other,
        zbus::Error::NameTaken => std::io::ErrorKind::Other,
        zbus::Error::InvalidMatchRule => std::io::ErrorKind::Other,
        zbus::Error::Failure(_) => std::io::ErrorKind::Other,
        zbus::Error::MissingParameter(_) => std::io::ErrorKind::Other,
        zbus::Error::InvalidSerial => std::io::ErrorKind::Other,
        zbus::Error::InterfaceExists(_, _) => std::io::ErrorKind::Other,
        _ => std::io::ErrorKind::Other,
    }
}
