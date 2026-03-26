# FLUX: Anti-EW Traffic Obfuscator

**FLUX** is a high-performance UDP proxy designed to defeat Electronic Warfare (EW) traffic analysis.

In Electronic Warfare (EW), adversaries don't need to decrypt your packets to know what you are doing. They use Traffic Analysis. If a drone sends a 50-byte packet every second, it's loitering. If it suddenly blasts a 2 MB stream, it just found a target. They use this metadata to direction-find (DF) your C2 or gateway and bomb it. 

FLUX solves this by forcing all your telemetry into a strict Constant Bit Rate (CBR) stream that looks identical to background RF noise.

### Not Just Obfuscation
FLUX operates as a complete, secure transport layer. For every packet that enters the pipeline, FLUX:
1. **Encodes** the data using a 2+1 Systematic XOR Forward Error Correction (FEC) scheme.
2. **Pads** the frame to a strict, fixed byte size using cryptographic white noise.
3. **Encrypts (AEAD)** and authenticates the entire payload and metadata headers in-place using XChaCha20-Poly1305.
4. **Transmits** at a hardcoded, unyielding frequency (e.g., 20Hz). If the drone has no telemetry to send, FLUX transmits dummy packets made entirely of cryptographic noise to maintain the illusion.


### Forward Error Correction (FEC)
Contested RF environments are inherently lossy. Because of the built-in zero-latency FEC layer, **up to 30% of your packets can be dropped by EW jammers, and the Gateway Node will still instantly mathematically reconstruct the missing data** on the receiving side without requiring a retransmission.

---

### Architecture
* **Transmitter (Edge Node):** Runs on the drone/robot. Ingests local plaintext UDP telemetry, processes it through the FLUX pipeline, and blasts it over the RF link.
* **Gateway Node:** Runs at your base station. Ingests the raw FLUX stream from the RF link, drops malformed packets/probes, verifies the XChaCha20 MAC, recovers dropped packets via FEC, and passes the clean, plaintext telemetry to your actual C2 server or receiver.

---

### Quick Start

**Build**
```bash
cargo build --release
```

Both nodes require a shared 32-byte cryptographic key passed strictly via environment variables.

**1. Generate and set the shared secret:**
Run this on **one** machine to generate the key:
```bash
openssl rand -hex 32
```
Copy the output, and export it as an environment variable on **BOTH** the Gateway and the Edge node:
```bash
export FLUX_KEY="<PASTE_THE_64_CHARACTER_STRING_HERE>"
```

**2. Start the Gateway Node (Base Station):**
```bash
# Listens for the encrypted FLUX stream on a port (e.g. 8888)
./target/release/receiver --listen 0.0.0.0:8888 --packet-size 1024
```

**3. Start the Transmitter (Drone / Edge Node):**
```bash
# Ingests plaintext telemetry on a local port (e.g. 7777), blasts obfuscated stream to the Gateway
./target/release/flux --listen 127.0.0.1:7777 --target <GATEWAY_IP>:8888 --packet-size 1024 --rate-hz 20
```

**4. Send Test Telemetry (On the Edge Node):**
Open a second terminal on your Edge Node (ensure `FLUX_KEY` is exported here too) and pipe plaintext UDP into the Transmitter:
```bash
echo "UAV_04_STATE: BINGO_FUEL" | nc -u -q 0 127.0.0.1 7777
```

*(Note: The `--packet-size` must perfectly match between the Transmitter and Gateway, or the packets will be mathematically rejected by the receiver's AEAD cipher).*


## Protocol Architecture

Every FLUX frame is identically sized and structured as follows:

```text
+-------------------+-----------------+----------------+-------------------------+------------------+
| Nonce (24 bytes)  | Seq # (4 bytes) | Len (2 bytes)  | Payload + Noise Padding | Poly1305 Tag (16)|
+-------------------+-----------------+----------------+-------------------------+------------------+
| Plaintext         | <--------------------------- ENCRYPTED -------------------------------------> |
+-------------------+-------------------------------------------------------------------------------+
```
* **Nonce:** 24-byte cryptographically secure random nonce (XChaCha20).
* **Seq #:** 4-byte sequence number used by the FEC engine to track dropped shards. (Dummy packets have `0`).
* **Len:** 2-byte Big Endian length of the *actual* payload. (Dummy packets have `0`).
* **Tag:** 16-byte Poly1305 authentication tag.

## License

MIT License - Created by [Viyle Technologies](https://viyle.com)
