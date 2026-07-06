# lan_drop

Zero-config file transfer over LAN, built in Rust on raw TCP and UDP sockets — no external networking libraries.

Point it at a file, and it finds receivers on the local network by itself: receivers broadcast their presence over UDP, senders discover them, and the file moves over a plain TCP stream with SHA-256 integrity verification at the end. No accounts, no config files, no cloud.

This is my final year BCA project and my first major Rust build. It has been through three full rewrites — the history is preserved as branches (see [Version history](#version-history)).

A note on how it was built: **every line of code across all three versions is hand-written**. I used AI for architectural discussions, guidance, and review-style polish suggestions, but none of the code is generated. The Huffman coder alone took a week of understanding, implementing, and refining — slower than prompting for it, but learning the language and the fundamentals was the whole point of this project. (This README, however, was AI-generated. I wrote the sockets, not the marketing.)

## How it works

```
 Receiver                                   Sender
 ────────                                   ──────
 TCP listener on <port>                     lan-cli discover
        │                                        │
        ├── UDP broadcast every 2s ──────────▶ live peer table
        │   "LAN_DROP|ip_addr=..|port=.."        │
        │                                        │
        ◀────────── TCP connect ─────────────────┤
        │                                        │
        ◀── metadata (bincode: name, size, ──────┤
        │    chunk size)                         │
        ◀── file bytes in 256 KiB chunks ────────┤
        │    (both sides hash-chain each chunk)  │
        ◀── 32-byte final digest ────────────────┤
        │
        digest match → rename .tmp → done
        digest mismatch → delete .tmp → "File corrupted!"
```

- **Discovery** — receivers advertise over UDP broadcast (`255.255.255.255:8787`); `discover` listens on the same port and maintains a live-updating peer table in the terminal, expiring peers not seen for 2 seconds.
- **Transfer** — a small hand-rolled protocol over TCP: a 4-byte length prefix, `bincode`-serialized metadata, then the raw file streamed in chunks through a `BufReader`.
- **Integrity** — both sides build a running SHA-256 hash chain over the chunks (a Merkle–Damgård-style construction). The sender ships its final digest after the file; if the receiver's digest doesn't match, the partial file is discarded.
- **Safety** — the receiver writes to a `.tmp` file and only renames it to the real filename after the digest check passes, so a failed transfer never leaves a half-written file behind.
- **Concurrency** — a from-scratch thread pool (no `rayon`, no `tokio`) runs the receiver and advertiser — or the discovery listener and peer-table writer — concurrently in a single process, coordinated over `mpsc` channels.

## Usage

Receive (advertises itself until a transfer completes):

```sh
cargo run -p lan-cli -- receive -p 9000 -f D:/downloads received_file.txt
```

Discover receivers on the network:

```sh
cargo run -p lan-cli -- discover
```

Send a file to a discovered receiver:

```sh
cargo run -p lan-cli -- send 192.168.1.42 9000 D:/files/video.mp4
```

Works across machines on the same LAN, or between two terminals on one machine for a quick test.

## Workspace layout

| Crate | Role |
|---|---|
| `lan-cli` | Command-line interface — argument parsing and dispatch |
| `lan-core` | Networking — UDP advertise/discovery, TCP sender/receiver, transfer protocol |
| `lan-engine` | Algorithms — SHA-256 hash chaining, Merkle tree, Huffman coding, thread pool |

## Version history

The project was rewritten from scratch twice, and each version lives on its own branch:

- [`v1`](../../tree/v1) — single-threaded and deliberately simple. Worked for direct transfers, but fell over as soon as advertise/discovery was added: one thread can't listen and transfer at the same time.
- [`v2`](../../tree/v2) — solved v1's problem with multiple processes. It worked correctly, but spawning several console windows per transfer was not acceptable, so it was scrapped.
- [`v3`](../../tree/v3) (current, merged to `main`) — single process, custom thread pool, and the codebase split into a proper three-crate workspace. Chunk hashing, a Merkle tree, and Huffman coding were added in `lan-engine` to satisfy university algorithm requirements — the per-chunk hashing costs some throughput, which is the main thing I want to win back.

## What I'd do differently

Things I know about and plan to fix as I keep working on this:

- **Frame the final digest properly.** The sender currently sleeps 200 ms after the file bytes before sending the digest, relying on timing instead of length-framing the trailer. A real protocol would delimit it explicitly.
- **Cut the per-chunk overhead.** Hashing every chunk plus logging progress on every read adds measurable latency on large files; the hashing should move off the hot path (or into the thread pool).
- **Error handling.** Parts of the code are still `unwrap()`-heavy from the "get sockets working first" phase and should propagate errors instead.
- **Finish wiring `lan-engine` in.** The Merkle tree and Huffman coding are implemented but not yet fully integrated into the transfer path — chunked verification (resume support) and optional compression are the goal.
- **Evaluate async.** The custom thread pool was a deliberate learning choice; a future version may compare it against a `tokio` implementation.

## Building

```sh
cargo build --release
cargo test
```

Developed and tested on Windows; the path handling normalizes Windows-style paths, and the networking is plain std sockets, so Linux/macOS should work but haven't been exercised much yet.
