// cargo build --release && scp ./target/release/pwr-chk root@192.168.0.99:/opt/bin

//scp ./systemd/pwrchk.service root@192.168.0.99:/lib/systemd/system
//systemctl enable pwrchk
//systemctl start pwrchk

use clap::Parser;
use tokio::{signal, sync::watch};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[clap(author, version = VERSION, about, long_about = None)]
struct AppParams {
    #[clap(short, long)]
    ping_ip: std::net::IpAddr,
    #[clap(short, long, default_value_t = 90)]
    delay_s: u64,
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> anyhow::Result<()> {
    let args = AppParams::parse();

    let (sender, mut receiver) = watch::channel(());
    let ip_str = args.ping_ip.to_string();
    println!("ping {}. poweroff after {} .", ip_str, args.delay_s);
    tokio::spawn(async move {
        let mut ping_ok = std::time::Instant::now();
        loop {
            tokio::select! {
                _ = receiver.changed() => {
                    break;
                },
                ping = check(&ip_str) => {
                    if ping {
                        ping_ok = std::time::Instant::now();
                    }
                }
            }
            if ping_ok.elapsed().as_secs() > args.delay_s {
                println!("ping timeout");
                break;
            }
            //println!("NO ping : {}s", ping_ok.elapsed().as_secs_f32());
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
        println!("BY-BY");
        tokio::process::Command::new("poweroff")
            .spawn()
            .expect("can't run poweroff")
            .wait()
            .await
    });
    wait_for_shutdown_signal().await;
    sender.send(())?;
    Ok(())
}

async fn check<R: AsRef<str>>(ip: R) -> bool {
    match tokio::process::Command::new("ping")
        .args(["-W", "0.5", "-c", "1"])
        .arg(ip.as_ref())
        .stdout(std::process::Stdio::null())
        .spawn()
    {
        Err(e) => {
            eprintln!("{}[{}:{}]", e, file!(), line!());
            false
        }
        Ok(mut child) => match child.wait().await {
            Err(e) => {
                eprintln!("{}[{}:{}]", e, file!(), line!());
                false
            }
            Ok(status) => status.code().unwrap_or(0) == 0,
        },
    }
}

#[allow(clippy::redundant_pub_crate)]
async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install ctrl+c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => println!("ctrl+c received, shutting down"),
        _ = terminate => println!("terminate received, shutting down"),
    }
}
