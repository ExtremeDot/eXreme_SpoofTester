use std::fs::{File, OpenOptions};
use std::io::{Write, BufReader};
use std::net::Ipv4Addr;
use std::time::Duration;
use std::collections::HashSet;
use chrono::Local;
use colored::*;
use ipnetwork::Ipv4Network;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::Packet;
use pnet::transport::{transport_channel, TransportChannelType};
use serde::Deserialize;
use clap::{Parser, ValueEnum};

const BANNER: &str = r#"
███████  ██   ██  ████████  ██████   ███████  ███    ███  ███████ 
██        ██ ██      ██     ██   ██  ██       ████  ████  ██      
█████      ███       ██     ██████   █████    ██ ████ ██  █████   
██        ██ ██      ██     ██   ██  ██       ██  ██  ██  ██      
███████  ██   ██     ██     ██   ██  ███████  ██      ██  ███████ 
           ⚡ EXTREME SPOOF TESTER - FINGLISH SMART EDITION ⚡
"#;

#[derive(Parser, Debug)]
#[command(
    name = "extreme_spoof_test",
    about = "Abzare pishrafte baraye teste amniat va nfoozpaziri spoofing dar shabake",
    long_about = r#"
======================================================================
                     HELP & AMOZESHE MAZHABI (HELP)
======================================================================
In afzar baraye teste in ast ke aya datacenter ya providere shabakeye shoma
ejazeye kharojoie paketha ba IP haye jali (Spoofed) ra midahad ya kheir.

Formate faile config.json bayad be sorate zir bashad (IP ha nemone and):
{
  "vps_1": "192.0.2.10",
  "vps_2": "198.51.100.20",
  "test_count": 5,
  "test_targets": [
    "192.0.2.0/24",
    "203.0.113.5"
  ]
}

RAHNAYI EJRA:
1. Dar sarvare girandeh (Masalan vps_1):
   sudo ./extreme_spoof_test r

2. Dar sarvare ferestandeh (Masalan vps_2):
   sudo ./extreme_spoof_test s

Narm afzar khodkar IP in system ra tashkhis dade va rahnayi lazem ra mikonad.
======================================================================"#
)]
struct Args {
    #[arg(value_enum, help = "S baraye Sender (Ferestandeh) va R baraye Receiver (Girandeh)")]
    mode: Mode,
    #[arg(short, long, default_value = "config.json", help = "Masire faile تنظیمات json")]
    config: String,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum Mode { S, R }

#[derive(Deserialize, Debug)]
struct Config {
    vps_1: Ipv4Addr,
    vps_2: Ipv4Addr,
    test_count: usize,
    test_targets: Vec<String>,
}

fn main() {
    println!("{}", BANNER.cyan().bold());
    let args = Args::parse();

    let config_file = File::open(&args.config).unwrap_or_else(|_| {
        eprintln!("{}", format!("❌ ERROR: Faile تنظیمات dar masire '{}' peyda nashod!", args.config).red());
        std::process::exit(1);
    });
    
    let config: Config = serde_json::from_reader(BufReader::new(config_file)).unwrap_or_else(|_| {
        eprintln!("{}", "❌ ERROR: Formate faile config.json eshtebah ast! Lutfan help ra check konid.".red());
        std::process::exit(1);
    });

    // Diagnosing roles safely without exposing real private network IPs
    let (my_ip, opposite_ip) = detect_vps_roles(&config);

    println!("📍 IP in System: {}", my_ip.to_string().green().bold());
    println!("🎯 IP Sarvare Moghabel: {}", opposite_ip.to_string().yellow().bold());

    let mut targets = Vec::new();
    for t in &config.test_targets {
        if let Ok(net) = t.parse::<Ipv4Network>() { targets.extend(net.iter()); }
        else if let Ok(ip) = t.parse::<Ipv4Addr>() { targets.push(ip); }
    }

    match args.mode {
        Mode::S => {
            run_sender(opposite_ip, config.test_count, targets);
        }
        Mode::R => {
            println!("\n📢 RAHNAYI HOOSHMAND:");
            println!("   Lutfan be sarvare moghabel ({}) beravid va in dastor ra ejra konid:", opposite_ip.to_string().cyan());
            println!("   {}", "sudo ./extreme_spoof_test s".bold().white());
            println!("{}", "--------------------------------------------------".dimmed());
            
            run_receiver(my_ip);
        }
    }
}

fn detect_vps_roles(config: &Config) -> (Ipv4Addr, Ipv4Addr) {
    let interfaces = pnet::datalink::interfaces();
    let mut local_ips = HashSet::new();

    for iface in interfaces {
        for ip_network in iface.ips {
            if let std::net::IpAddr::V4(ipv4) = ip_network.ip() {
                local_ips.insert(ipv4);
            }
        }
    }

    if local_ips.contains(&config.vps_1) {
        (config.vps_1, config.vps_2)
    } else if local_ips.contains(&config.vps_2) {
        (config.vps_2, config.vps_1)
    } else {
        println!("{}", "⚠️ WARNING: IP mahali ba vps_1 ya vps_2 motabeghat nadarad. Pishfarz vps_1 entekhab shod.".yellow());
        (config.vps_1, config.vps_2)
    }
}

fn run_sender(receiver_ip: Ipv4Addr, test_count: usize, targets: Vec<Ipv4Addr>) {
    println!("{}", "🚀 Dar halat amade saziye pakethaye spoof shode (ICMP)...".yellow());
    
    let (mut tx, _) = transport_channel(4096, TransportChannelType::Layer3(IpNextHeaderProtocols::Icmp))
        .unwrap_or_else(|e| {
            eprintln!("{}", format!("❌ ERROR: Khata dar baz kardane socket! Az 'sudo' estefade konid. Internal: {}", e).red());
            std::process::exit(1);
        });

    println!("{}", format!("📊 Tedade koll target IP ha baraye spoof: {}", targets.len()).green());
    println!("{}", "--------------------------------------------------".dimmed());

    let mut buffer = [0u8; 40];
    let mut total_sent = 0;

    for ip in targets {
        for _ in 0..test_count {
            let mut packet = MutableIpv4Packet::new(&mut buffer).unwrap();
            packet.set_version(4);
            packet.set_header_length(5);
            packet.set_total_length(40);
            packet.set_ttl(64);
            packet.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
            packet.set_source(ip); 
            packet.set_destination(receiver_ip); 
            packet.set_payload(b"SPOOF_TEST_DATA_PING");

            let _ = tx.send_to(packet, std::net::IpAddr::V4(receiver_ip));
            total_sent += 1;
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    
    println!("{}", "--------------------------------------------------".dimmed());
    println!("{}", format!("✅ AMALIAT ERSAL TAMAM! Total {} paketh ghaele ghabol be ({}) ferestade shod.", total_sent, receiver_ip).green().bold());
}

fn run_receiver(my_ip: Ipv4Addr) {
    println!("{}", "🎧 Dar hale goosh dadan be pakethaye spoof shode (Live Feed)...".magenta());

    let (_, mut rx) = transport_channel(4096, TransportChannelType::Layer3(IpNextHeaderProtocols::Icmp))
        .unwrap_or_else(|e| {
            eprintln!("{}", format!("❌ ERROR: Khata dar capturing! Az 'sudo' estefade konid. Internal: {}", e).red());
            std::process::exit(1);
        });

    let filename = format!("working_ips_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
    let mut file = OpenOptions::new().create(true).write(true).append(true).open(&filename).unwrap();
    
    println!("{}", format!("📝 IP haye movafagh dar in faile zakhire mishavand: {}", filename).blue());
    println!("{}", "\n┌──────────────────────────────────────────────┐".dimmed());
    println!("│ {} │", "     DETECTED OPEN SPOOF CHANNELS      ".bold().cyan());
    println!("{}", "└──────────────────────────────────────────────┘".dimmed());

    let mut iter = pnet::transport::ipv4_packet_iter(&mut rx);
    let mut discovered_ips = HashSet::new();

    while let Ok((packet, _)) = iter.next() {
        if packet.get_destination() == my_ip && packet.get_next_level_protocol() == IpNextHeaderProtocols::Icmp {
            if packet.payload().contains(&b'S') && packet.payload().contains(&b'P') {
                let spoofed_ip = packet.get_source();
                
                if discovered_ips.insert(spoofed_ip) {
                    let current_time = Local::now().format("%H:%M:%S").to_string();
                    
                    println!(
                        "{} {} {}",
                        format!("[{}]", current_time).dimmed(),
                        "🔥 [VULNERABLE]".green().bold(),
                        format!("Spoof Allowed From IP: {}", spoofed_ip).white()
                    );
                    
                    writeln!(file, "{}", spoofed_ip).unwrap();
                }
            }
        }
    }
}
