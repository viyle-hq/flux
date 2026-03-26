use crate::types::FluxPayload;
use chacha20poly1305::{
    XChaCha20Poly1305,
    aead::{AeadCore, AeadInPlace, KeyInit, OsRng},
};
use rand::{RngCore, thread_rng};

pub const NONCE_SIZE: usize = 24;
pub const TAG_SIZE: usize = 16;
pub const SEQ_SIZE: usize = 4;
pub const LEN_SIZE: usize = 2;
pub const PROTOCOL_OVERHEAD: usize = NONCE_SIZE + TAG_SIZE + SEQ_SIZE + LEN_SIZE;

pub struct Framer {
    packet_size: usize,
    cipher: XChaCha20Poly1305,
}

impl Framer {
    pub fn new(packet_size: usize, key: &[u8; 32]) -> Self {
        assert!(
            packet_size > PROTOCOL_OVERHEAD,
            "Packet size too small for crypto overhead"
        );
        Self {
            packet_size,
            cipher: XChaCha20Poly1305::new(key.into()),
        }
    }

    pub fn pack(&self, seq: u32, payload: FluxPayload, out_buffer: &mut [u8]) {
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        out_buffer[..NONCE_SIZE].copy_from_slice(&nonce);

        let pt_area = &mut out_buffer[NONCE_SIZE..self.packet_size - TAG_SIZE];
        thread_rng().fill_bytes(pt_area);

        match payload {
            FluxPayload::Data(data) => {
                let len = data.len() as u16;
                pt_area[0..4].copy_from_slice(&seq.to_be_bytes());
                pt_area[4..6].copy_from_slice(&len.to_be_bytes());
                pt_area[6..6 + data.len()].copy_from_slice(data);
            }
            FluxPayload::Dummy => {
                pt_area[0..4].copy_from_slice(&0u32.to_be_bytes());
                pt_area[4..6].copy_from_slice(&0u16.to_be_bytes());
            }
        }

        let tag = self
            .cipher
            .encrypt_in_place_detached(&nonce, b"", pt_area)
            .expect("Encryption failure");

        out_buffer[self.packet_size - TAG_SIZE..].copy_from_slice(&tag);
    }
}
