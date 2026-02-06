use crate::models::clients::Client;

pub fn active_clients() -> Option<Vec<Client>> {
    let output = std::process::Command::new("hyprctl")
        .args(["clients", "-j"])
        .output()
        .expect("Failed to execute command");

    let json_str = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&json_str).expect("Error al parsear JSON")
}

pub fn focus_client(client: &Client) {
    std::process::Command::new("hyprctl")
        .args([
            "dispatch",
            "focuswindow",
            format!("class:^{}$", client.class).as_str(),
        ])
        .output()
        .expect("Failed to execute command");
}

#[cfg(test)]
mod clients_tests {
    use super::*;

    #[test]
    fn should_return_a_vec_of_active_clients() {
        let clients = active_clients();
        assert!(clients.is_some())
    }
}
