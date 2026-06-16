use clap::{Args, Parser};
use std::io;
use std::process::Command;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Configure DNS for NetworkManager connections"
)]
struct Cli {
    #[command(flatten)]
    config: Config,
}

#[derive(Args)]
struct Config {
    /// DNS servers to set (space separated)
    #[arg(short, long, default_value = "10.0.0.144 1.1.1.1 1.0.0.1")]
    dns: String,

    /// Specific connection name to modify
    #[arg(short, long)]
    connection: Option<String>,

    /// Apply to all active connections
    #[arg(short, long)]
    all: bool,

    /// Show changes without applying them
    #[arg(short = 'r', long)]
    dry_run: bool,
}

fn get_active_connections() -> io::Result<Vec<String>> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "NAME", "connection", "show", "--active"])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .filter(|line| {
            let l = line.to_lowercase();
            !l.contains("lo") && !l.contains("docker0") && !l.contains("docker")&& !l.contains("bridge") && !l.contains("nat")&& !l.contains("host") && !l.contains("none")&& !l.contains("br-")
        })
        .map(|s| s.to_string())
        .collect())
}

fn modify_connection_dns(connection: &str, dns: &str, dry_run: bool) -> io::Result<()> {
    println!("Updating connection: {}", connection);

    if dry_run {
        println!(
            "[Dry Run] Would run: nmcli connection modify {} ipv4.dns \"{}\" ipv4.ignore-auto-dns yes",
            connection, dns
        );
        return Ok(());
    }

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
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let up_output = Command::new("nmcli")
        .args(["connection", "up", connection])
        .output()?;

    if !up_output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            String::from_utf8_lossy(&up_output.stderr).to_string(),
        ));
    }

    println!("Successfully updated and reactivated {}!", connection);
    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let dns = &cli.config.dns;
    let dry_run = cli.config.dry_run;

    if let Some(conn) = cli.config.connection {
        modify_connection_dns(&conn, dns, dry_run)?;
    } else if cli.config.all {
        let active = get_active_connections()?;
        if active.is_empty() {
            println!("No active connections found.");
        } else {
            for conn in active {
                modify_connection_dns(&conn, dns, dry_run)?;
            }
        }
    } else {
        println!("Please specify a connection (-c) or use --all (-a).");
        std::process::exit(1);
    }

    Ok(())
}
