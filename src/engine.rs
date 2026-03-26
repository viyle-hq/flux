use crate::fec::FecEncoder;
use crate::framer::Framer;
use crate::types::FluxPayload;
use std::net::UdpSocket;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

pub fn run_metronome(
    rx: Receiver<Vec<u8>>,
    target: &str,
    packet_size: usize,
    rate_hz: u64,
    key: &[u8; 32],
) {
    let outbound_socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind outbound socket");
    outbound_socket
        .connect(target)
        .expect("Failed to connect to target");

    let framer = Framer::new(packet_size, key);
    let mut fec = FecEncoder::new();

    let tick_duration = Duration::from_micros(1_000_000 / rate_hz);
    let mut next_tick = Instant::now() + tick_duration;

    let mut out_buffer = vec![0u8; packet_size];
    let mut last_real_packet = Instant::now();
    let mut warned_idle = false;
    let mut queued_parity: Option<(u32, Vec<u8>)> = None;

    loop {
        if let Some((p_seq, p_data)) = queued_parity.take() {
            framer.pack(p_seq, FluxPayload::Data(&p_data), &mut out_buffer);
        } else {
            match rx.try_recv() {
                Ok(data) => {
                    last_real_packet = Instant::now();
                    warned_idle = false;

                    let (seq, parity) = fec.encode(&data);
                    queued_parity = parity;

                    framer.pack(seq, FluxPayload::Data(&data), &mut out_buffer);
                }
                Err(_) => {
                    if last_real_packet.elapsed().as_secs() >= 45 && !warned_idle {
                        println!("[WARN] No telemetry in 45s. Transmitting purely noise.");
                        warned_idle = true;
                    }
                    framer.pack(0, FluxPayload::Dummy, &mut out_buffer);
                }
            }
        }

        let _ = outbound_socket.send(&out_buffer);

        let now = Instant::now();
        if now < next_tick {
            thread::sleep(next_tick - now);
        }
        next_tick += tick_duration;
    }
}
