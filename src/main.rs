use std::process::Command;

fn modify_connection_dns(connection: &str, dns: &str) {
    let output = Command::new("nmcli")
        .args([
            "connection",
            "modify",
            connection,
            "ipv4.dns",
            dns,
            "ipv4.ignore-auto-dns",
            "yes",
        ])
        .output()
        .expect("Failed to execute nmcli command");

    if output.status.success() {
        println!("Successfully changed DNS!")
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error changing DNS: {}", stderr);
    }
}

fn main() {
    let connection_name = "Mtrainer5";
    let dns_servers = "10.0.0.144 1.1.1.1 1.0.0.1";

    modify_connection_dns(connection_name, dns_servers);

    Command::new("nmcli").args(["connection", "up", "Mtrainer5"]);
}
