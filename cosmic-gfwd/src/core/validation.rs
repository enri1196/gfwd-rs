pub fn validate_interface_name(interface: &str) -> Result<(), String> {
    let interface = interface.trim();

    if interface.is_empty() {
        return Err("Interface name is required".to_string());
    }

    if interface.len() > 15 {
        return Err("Interface name must be 15 characters or fewer".to_string());
    }

    if !interface
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
    {
        return Err(
            "Interface name may only contain letters, numbers, dashes, underscores, dots, and colons"
                .to_string(),
        );
    }

    Ok(())
}
