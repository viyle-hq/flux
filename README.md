# FLUX: Anti-EW UDP Tunnel

**FLUX** is a zero-pattern UDP proxy designed to defeat Electronic Warfare (EW) traffic analysis.

## The Problem
In Electronic Warfare (EW), adversaries don't need to decrypt your packets to know what you are doing. They do Traffic Analysis, often using AI and Machine Learning. If a drone sends a 50-byte packet every second, it's loitering. If it suddenly blasts a 2 MB stream, it just found a target. They use this data to direction-find (DF) your C2 or gateway and bomb it. 

FLUX solves this by flattening your transmission footprint into a continuous, zero-variance stream of cryptographic noise. To  
an adversary, your behavior never changes.

### Defense-in-Depth Transport

FLUX provides a self-contained, end-to-end secure transport layer.

For every packet, it handles:
1. **FEC Encoding:** Uses a 2+1 Systematic XOR Forward Error Correction scheme. Up to 30% of your packets can be dropped by EW jammers, and the Gateway will instantly mathematically reconstruct the missing data without retransmissions.
2. **Padding:** Forces the frame to a strict, fixed byte size using cryptographic white noise.
3. **Encryption (AEAD):** Authenticates and encrypts the entire payload and metadata headers in-place using XChaCha20-Poly1305.
4. **CBR Transmission:** Blasts packets at a hardcoded, unyielding frequency (e.g., 20Hz). Idle time is filled entirely with dummy noise packets.

### Deployment Model
* **Transmitter (Edge):** Runs on the drone/robot. Ingests local plaintext UDP telemetry, processes it through the FLUX pipeline, and blasts it over the RF link.
* **Gateway (Base):** Runs at your base station. Ingests the raw FLUX stream, verifies the MAC (dropping malformed probes), recovers dropped packets via FEC, and passes clean plaintext to your C2 server.
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


### Protocol Architecture

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
