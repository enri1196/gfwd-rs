use std::collections::HashMap;

use zbus::Result as ZResult;
use zbus::Connection;
use zbus_macros::proxy;
use zvariant::OwnedValue;

#[proxy(
    interface = "org.fedoraproject.FirewallD1",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1",
    default_path = "/org/fedoraproject/FirewallD1"
)]
/// Proxy for the `org.fedoraproject.FirewallD1` interface
pub trait FirewallD1 {
    /// Inizia l’autorizzazione completa su Firewalld (per app di configurazione).
    #[zbus(name = "authorizeAll")]
    fn authorize_all(&self) -> ZResult<()>;

    /// Ricarica completamente il firewall (perdita di state, terminate connessioni).
    #[zbus(name = "completeReload")]
    fn complete_reload(&self) -> ZResult<()>;

    /// Disabilita panic mode.
    #[zbus(name = "disablePanicMode")]
    fn disable_panic_mode(&self) -> ZResult<()>;

    /// Abilita panic mode (drop di tutti i pacchetti).
    #[zbus(name = "enablePanicMode")]
    fn enable_panic_mode(&self) -> ZResult<()>;

    /// Restituisce la zona di default.
    #[zbus(name = "getDefaultZone")]
    fn get_default_zone(&self) -> ZResult<String>;

    /// Elenca tutti i servizi in runtime.
    #[zbus(name = "listServices")]
    fn list_services(&self) -> ZResult<Vec<String>>;

    /// Restituisce le impostazioni di un servizio (chiave→variante).
    #[zbus(name = "getServiceSettings2")]
    fn get_service_settings2(
        &self,
        service: &str,
    ) -> ZResult<HashMap<String, OwnedValue>>;

    /// Ricarica le regole (keep state).
    #[zbus(name = "reload")]
    fn reload(&self) -> ZResult<()>;

    /// Trasforma le impostazioni runtime in permanenti.
    #[zbus(name = "runtimeToPermanent")]
    fn runtime_to_permanent(&self) -> ZResult<()>;

    /// Controlla la configurazione permanente.
    #[zbus(name = "checkPermanentConfig")]
    fn check_permanent_config(&self) -> ZResult<()>;

    /// Imposta la zona di default (runtime + permanente).
    #[zbus(name = "setDefaultZone")]
    fn set_default_zone(&self, zone: &str) -> ZResult<()>;

    /// Imposta il livello di log denials (all, unicast, …, off).
    #[zbus(name = "setLogDenied")]
    fn set_log_denied(&self, value: &str) -> ZResult<()>;

    /// Emesso quando cambia la default zone.
    #[zbus(signal, name = "DefaultZoneChanged")]
    fn default_zone_changed(&self, zone: &str) -> ZResult<()>;

    /// Emesso quando cambia LogDenied.
    #[zbus(signal, name = "LogDeniedChanged")]
    fn log_denied_changed(&self, value: &str) -> ZResult<()>;

    /// Emesso quando panic mode viene disattivato.
    #[zbus(signal, name = "PanicModeDisabled")]
    fn panic_mode_disabled(&self) -> ZResult<()>;

    /// Emesso quando panic mode viene attivato.
    #[zbus(signal, name = "PanicModeEnabled")]
    fn panic_mode_enabled(&self) -> ZResult<()>;

    /// Emesso su ogni reload (incluso completeReload).
    #[zbus(signal, name = "Reloaded")]
    fn reloaded(&self) -> ZResult<()>;

    #[zbus(property, name = "BRIDGE")]
    fn bridge(&self) -> ZResult<bool>;

    #[zbus(property, name = "IPSet")]
    fn ip_set(&self) -> ZResult<bool>;

    #[zbus(property, name = "IPSetTypes")]
    fn ip_set_types(&self) -> ZResult<Vec<String>>;

    #[zbus(property, name = "IPv4")]
    fn ipv4(&self) -> ZResult<bool>;

    #[zbus(property, name = "IPv4ICMPTypes")]
    fn ipv4_icmp_types(&self) -> ZResult<Vec<String>>;

    #[zbus(property, name = "IPv6")]
    fn ipv6(&self) -> ZResult<bool>;

    #[zbus(property, name = "IPv6_rpfilter")]
    fn ipv6_rpfilter(&self) -> ZResult<bool>;

    #[zbus(property, name = "IPv6ICMPTypes")]
    fn ipv6_icmp_types(&self) -> ZResult<Vec<String>>;
}

/// Crea un proxy già connesso al bus di sistema
pub async fn new_firewalld_proxy() -> ZResult<FirewallD1Proxy<'static>> {
    let conn = Connection::system().await?;
    FirewallD1Proxy::<'static>::new(&conn).await
}
