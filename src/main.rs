use clap::Parser;
use flux::engine;
use std::env;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "FLUX: Anti-EW Traffic Obfuscator (Transmitter)"
)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1:8000")]
    listen: String,
    #[arg(short, long)]
    target: String,
    #[arg(short = 's', long, default_value_t = 1024)]
    packet_size: usize,
    #[arg(short = 'r', long, default_value_t = 50)]
    rate_hz: u64,
}

fn get_secret_key() -> [u8; 32] {
    let key_hex =
        env::var("FLUX_KEY").expect("FLUX_KEY env var not set. Export a 64-char hex string.");
    let decoded = hex::decode(key_hex.trim()).expect("FLUX_KEY must be valid hex");
    assert_eq!(decoded.len(), 32, "FLUX_KEY must be exactly 32 bytes");
    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    key
}

fn main() {
    let args = Args::parse();
    let key = get_secret_key();

    println!("[INFO] FLUX Transmitter Started");
    println!(
        "[INFO] Target: {} | CBR: {} bytes @ {}Hz",
        args.target, args.packet_size, args.rate_hz
    );

    let local_socket = UdpSocket::bind(&args.listen).expect("Failed to bind local listener");
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let packet_size = args.packet_size;

    thread::spawn(move || {
        let mut buffer = vec![0u8; 65535];
        loop {
            if let Ok((size, _)) = local_socket.recv_from(&mut buffer) {
                let max_payload = packet_size - flux::framer::PROTOCOL_OVERHEAD;
                let valid_size = size.min(max_payload);
                let _ = tx.send(buffer[..valid_size].to_vec());
            }
        }
    });

    engine::run_metronome(rx, &args.target, args.packet_size, args.rate_hz, &key);
}
