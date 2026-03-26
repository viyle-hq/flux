use clap::Parser;
use flux::{deframer::Deframer, fec::FecDecoder, types::DeframedPayload};
use std::env;
use std::net::UdpSocket;

#[derive(Parser, Debug)]
#[command(author, version, about = "FLUX: High-Performance P2P Receiver Node")]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0:9000")]
    listen: String,
    #[arg(short = 's', long, default_value_t = 1024)]
    packet_size: usize,
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

    let socket = UdpSocket::bind(&args.listen).expect("Failed to bind C2 receiver");
    let deframer = Deframer::new(args.packet_size, &key);
    let mut fec = FecDecoder::new();

    let mut rx_buffer = vec![0u8; args.packet_size];

    println!("[INFO] FLUX C2 Receiver Started on {}", args.listen);
    println!(
        "[INFO] Enforcing {} byte frames + FEC enabled...",
        args.packet_size
    );

    loop {
        if let Ok((size, addr)) = socket.recv_from(&mut rx_buffer) {
            let wire_frame = &mut rx_buffer[..size];

            match deframer.deframe(wire_frame) {
                Ok(DeframedPayload::Data { seq, payload }) => {
                    let recovered_payloads = fec.receive(seq, payload);

                    for (r_seq, r_data) in recovered_payloads {
                        match std::str::from_utf8(&r_data) {
                            Ok(text) => println!(
                                "[DATA from {}] (Seq {}) {} bytes: {}",
                                addr,
                                r_seq,
                                r_data.len(),
                                text.trim()
                            ),
                            Err(_) => println!(
                                "[DATA from {}] (Seq {}) {} bytes: <binary>",
                                addr,
                                r_seq,
                                r_data.len()
                            ),
                        }
                    }
                }
                Ok(DeframedPayload::Dummy) => {}
                Err(e) => {
                    eprintln!("[WARN] Drop ({addr}): {e}");
                }
            }
        }
    }
}
