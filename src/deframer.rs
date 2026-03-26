use crate::framer::{NONCE_SIZE, TAG_SIZE};
use crate::types::DeframedPayload;
use chacha20poly1305::{KeyInit, Tag, XChaCha20Poly1305, XNonce, aead::AeadInPlace};

pub struct Deframer {
    packet_size: usize,
    cipher: XChaCha20Poly1305,
}

impl Deframer {
    pub fn new(packet_size: usize, key: &[u8; 32]) -> Self {
        Self {
            packet_size,
            cipher: XChaCha20Poly1305::new(key.into()),
        }
    }

    pub fn deframe<'a>(
        &self,
        wire_data: &'a mut [u8],
    ) -> Result<DeframedPayload<'a>, &'static str> {
        if wire_data.len() != self.packet_size {
            return Err("Packet size mismatch");
        }

        let (nonce_bytes, rest) = wire_data.split_at_mut(NONCE_SIZE);
        let (ciphertext, tag_bytes) = rest.split_at_mut(rest.len() - TAG_SIZE);

        let nonce = XNonce::from_slice(nonce_bytes);
        let tag = Tag::from_slice(tag_bytes);

        self.cipher
            .decrypt_in_place_detached(nonce, b"", ciphertext, tag)
            .map_err(|_| "Auth failed")?;

        let seq = u32::from_be_bytes([ciphertext[0], ciphertext[1], ciphertext[2], ciphertext[3]]);
        let data_len = u16::from_be_bytes([ciphertext[4], ciphertext[5]]) as usize;

        if data_len == 0 {
            return Ok(DeframedPayload::Dummy);
        }

        if data_len + 6 > ciphertext.len() {
            return Err("Malformed internal header");
        }

        Ok(DeframedPayload::Data {
            seq,
            payload: &ciphertext[6..6 + data_len],
        })
    }
}
