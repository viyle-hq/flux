use std::collections::HashMap;

pub const FEC_K: u32 = 2;
pub const FEC_N: u32 = 3;

pub struct FecEncoder {
    buffer: Vec<Vec<u8>>,
    next_seq: u32,
}

impl Default for FecEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl FecEncoder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(FEC_K as usize),
            next_seq: 0,
        }
    }

    pub fn encode(&mut self, data: &[u8]) -> (u32, Option<(u32, Vec<u8>)>) {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.buffer.push(data.to_vec());

        if self.buffer.len() == FEC_K as usize {
            let parity_seq = self.next_seq;
            self.next_seq += 1;

            let max_len = self.buffer.iter().map(|b| b.len()).max().unwrap_or(0);
            let mut parity = vec![0u8; max_len];

            for buf in &self.buffer {
                for i in 0..buf.len() {
                    parity[i] ^= buf[i];
                }
            }

            self.buffer.clear();
            (seq, Some((parity_seq, parity)))
        } else {
            (seq, None)
        }
    }
}

pub struct FecDecoder {
    blocks: HashMap<u32, [Option<Vec<u8>>; 3]>,
}

impl Default for FecDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl FecDecoder {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    pub fn receive(&mut self, seq: u32, payload: &[u8]) -> Vec<(u32, Vec<u8>)> {
        let mut yielded = Vec::new();
        let block_id = seq / FEC_N;
        let shard_id = (seq % FEC_N) as usize;

        let block = self
            .blocks
            .entry(block_id)
            .or_insert_with(|| [None, None, None]);
        block[shard_id] = Some(payload.to_vec());

        if shard_id < FEC_K as usize {
            yielded.push((seq, payload.to_vec()));
        }

        let data_present = block[0..FEC_K as usize]
            .iter()
            .filter(|x| x.is_some())
            .count();
        let parity_present = block[FEC_N as usize - 1].is_some();

        if data_present == (FEC_K as usize - 1) && parity_present {
            let missing_idx = block[0..(FEC_K as usize)]
                .iter()
                .position(|x| x.is_none())
                .unwrap_or(0);

            let max_len = block.iter().flatten().map(|s| s.len()).max().unwrap_or(0);
            let mut recovered_shard = vec![0u8; max_len];

            for (i, shard) in block.iter().enumerate() {
                if i != missing_idx {
                    if let Some(s) = shard {
                        for j in 0..s.len() {
                            recovered_shard[j] ^= s[j];
                        }
                    }
                }
            }

            block[missing_idx] = Some(recovered_shard.clone());
            yielded.push((block_id * FEC_N + missing_idx as u32, recovered_shard));
        }

        self.blocks.retain(|&id, _| id + 10 >= block_id);

        yielded
    }
}