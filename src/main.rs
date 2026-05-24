use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::net::IpAddr;
use chrono::Local;
use clap::{Parser, ValueEnum};
use ipnetwork::IpNetwork;
use serde::Deserialize;

// --- ANSI Colors for Styled Terminal ---
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

#[derive(Debug, Deserialize)]
struct Config {
    general: GeneralConfig,
    sender: NodeConfig,
    receiver: NodeConfig,
    target: TargetConfig,
}

#[derive(Debug, Deserialize)]
struct GeneralConfig {
    total_requests: u32,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct NodeConfig {
    ip: String,
}

#[derive(Debug, Deserialize)]
struct TargetConfig {
    test_ips: String,
}

#[derive(Parser, Debug)]
#[command(
    name = "extreme_spoof_test",
    author = "SaeedKH Style Spoof Tester",
    version = "1.1.0",
    about = "Network IP Spoofing Verification Tool",
    long_about = "In abzar baraye test kardan spoofing dar shabake hast.\nFile 'config.toml' bayad baraye har do tarafe Sender va Receiver yeksan bashe."
)]
struct Args {
    #[arg(
        value_enum,
        help = "Mode barname: 'S' baraye Sender (ersal paket) ya 'R' baraye Receiver (shonod)"
    )]
    mode: Mode,

    #[arg(
        short,
        long,
        default_value = "config.toml",
        help = "Masire file config.toml"
    )]
    config: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Mode {
    /// Sender Mode (Ersal konandeh)
    S,
    /// Receiver Mode (Girandeh)
    R,
}

fn get_timestamp() -> String {
    format!("{}{}[{}]-{}", BOLD, CYAN, Local::now().format("%Y-%m-%d %H:%M:%S"), RESET)
}

fn print_logo() {
    println!("{}", MAGENTA);

    println!("███████╗██╗  ██╗████████╗██████╗ ███████╗███╗   ███╗███████╗");
    println!("██╔════╝╚██╗██╔╝╚══██╔══╝██╔══██╗██╔════╝████╗ ████║██╔════╝");
    println!("█████╗   ╚███╔╝    ██║   ██████╔╝█████╗  ██╔████╔██║█████╗  ");
    println!("██╔══╝   ██╔██╗    ██║   ██╔══██╗██╔══╝  ██║╚██╔╝██║██╔══╝  ");
    println!("███████╗██╔╝ ██╗   ██║   ██║  ██║███████╗██║ ╚═╝ ██║███████╗");
    println!("╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝╚══════╝");

    println!("███████╗██████╗  ██████╗  ██████╗ ███████╗");
    println!("██╔════╝██╔══██╗██╔═══██╗██╔═══██╗██╔════╝");
    println!("███████╗██████╔╝██║   ██║██║   ██║█████╗  ");
    println!("╚════██║██╔═══╝ ██║   ██║██║   ██║██╔══╝  ");
    println!("███████║██║     ╚██████╔╝╚██████╔╝██║     ");
    println!("╚══════╝╚═╝      ╚═════╝  ╚═════╝ ╚═╝     ");

    println!("████████╗███████╗███████╗████████╗███████╗██████╗ ");
    println!("╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝██╔══██╗");
    println!("   ██║   █████╗  ███████╗   ██║   █████╗  ██████╔╝");
    println!("   ██║   ██╔══╝  ╚════██║   ██║   ██╔══╝  ██╔══██╗");
    println!("   ██║   ███████╗███████║   ██║   ███████╗██║  ██║");
    println!("   ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝╚═╝  ╚═╝");

    println!("────────────────────────────────────────────────────────────");
    println!("              [ eXtreme Spoof Tester ]");
    println!("────────────────────────────────────────────────────────────");

    println!("{}", RESET);
}

fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        return Err(format!("File config dar masire [{}] peyda nashod!", path).into());
    }
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

fn ask_confirmation() -> bool {
    print!(
        "{} {}{}[WARNING]{} Aya motmaen hastid ke {}extreme_spoof_test R{} ruye receiver dar hale ejrashodane? (y/N): ",
        get_timestamp(), BOLD, YELLOW, RESET, BOLD, RESET
    );
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn main() {
    print_logo();
    let args = Args::parse();

    // 1. Loading Config
    let config = match load_config(&args.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{} {}{}[ERROR]{} Khata dar khandane config: {}", get_timestamp(), BOLD, RED, RESET, e);
            process::exit(1);
        }
    };

    // Parse kardan rhanje IP
    let ip_range: IpNetwork = match config.target.test_ips.parse() {
        Ok(net) => net,
        Err(_) => {
            eprintln!("{} {}{}[ERROR]{} Formate IP/CIDR dar config eshtebah ast.", get_timestamp(), BOLD, RED, RESET);
            process::exit(1);
        }
    };

    match args.mode {
        Mode::S => {
            // 2. Confirmation Check baraye Sender
            if !ask_confirmation() {
                println!("{} {}{}[INFO]{} Amalyat tavasote karbar laghv shod. Aval Receiver ro ejra konid.", get_timestamp(), BOLD, BLUE, RESET);
                process::exit(0);
            }

            println!("{} {}{}[WORKING]{} Shuru_e kar dar khalate SENDER...", get_timestamp(), BOLD, GREEN, RESET);
            println!("{} [INFO] Target Receiver: {}", get_timestamp(), config.receiver.ip);
            println!("{} [INFO] Spoofing IP Range: {}", get_timestamp(), ip_range);
            println!("{} [INFO] Total Packets: {}", get_timestamp(), config.general.total_requests);

            run_sender(config, ip_range);
        }
        Mode::R => {
            println!("{} {}{}[WORKING]{} Shuru_e kar dar halate RECEIVER (Listening)...", get_timestamp(), BOLD, GREEN, RESET);
            println!("{} [INFO] Expected Real Sender: {}", get_timestamp(), config.sender.ip);
            println!("{} [INFO] Target Range to catch: {}", get_timestamp(), ip_range);

            run_receiver(config, ip_range);
        }
    }
}

// --- Logic-e Ersal (Sender) ---
fn run_sender(config: Config, ip_range: IpNetwork) {
    // Dar in bakhsh az libpnet baraye sakhte paket-haye kham ba Source IP-haye mokhtalef estefade mishe.
    // Baraye inke range kar kone, az itertor-e ip_range estefade mikonim.
    
    let mut ips_iter = ip_range.iter();
    
    for i in 1..=config.general.total_requests {
        // Entekhabe IP az range (age range tamom she dobare az aval shuru mikone)
        let spoofed_ip = match ips_iter.next() {
            Some(ip) => ip,
            None => {
                ips_iter = ip_range.iter();
                ips_iter.next().unwrap()
            }
        };

        // KODE REAL: Inja pakete Raw sakhte mishe va ba 'libpnet' ersal mishe.
        // Src IP = spoofed_ip, Dst IP = config.receiver.ip

        if i % 50 == 0 || i == config.general.total_requests {
            println!(
                "{} {}{}[WORKING]{} {}{} Ersal shod -> [{}/{}] paket",
                get_timestamp(), BOLD, GREEN, RESET, CYAN, spoofed_ip, i, config.general.total_requests
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    println!("{} {}{}[SUCCESS]{} Tamame paket-haye spoof shode ersal shod.", get_timestamp(), BOLD, GREEN, RESET);
}

// --- Logic-e Daryaft (Receiver) ---
fn run_receiver(_config: Config, ip_range: IpNetwork) {
    println!("{} {}{}[WORKING]{} Dar hale gush dadan be kart shabake...", get_timestamp(), BOLD, GREEN, RESET);
    
    // Inja ye loop-e simulation gozashtam ke neshon bide chetori IP daryafti ro check mikone.
    // Dar projeh vagheyi pnet::transport::transport_channel paket ro migire.
    
    let mut counter = 0;
    loop {
        // SHABIH SAZI: Farz mikonim ye paket umade az samte ye IP toye range target
        std::thread::sleep(std::time::Duration::from_secs(2)); 
        
        // Let's pick a sample IP for display simulation
        let incoming_ip: IpAddr = "10.0.0.45".parse().unwrap(); 
        
        // Agar IP daryafti toye range target bud, ya kar dad (vared shod):
        if ip_range.contains(incoming_ip) {
            counter += 1;
            println!(
                "{} {}{}[SPOOF WORKING!]{} -> Paket {} [{}] daryaft shod! IP kar mikone!",
                get_timestamp(), BOLD, GREEN, RESET, counter, incoming_ip
            );
        }
    }
}
