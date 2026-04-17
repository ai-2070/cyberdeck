//! Synchronized mesh demo — runs on multiple machines, renders the same
//! animated ASCII plasma across all screens simultaneously.
//!
//! Each node discovers peers via UDP broadcast on the LAN. The animation
//! is computed locally from wall-clock time, so all machines with roughly
//! synced clocks produce the same frame. No central coordinator.
//!
//! Usage (run on each machine):
//!
//!   cargo run --release --example mesh_demo -- <name>
//!
//! Examples:
//!   cargo run --release --example mesh_demo -- laptop
//!   cargo run --release --example mesh_demo -- desktop
//!   cargo run --release --example mesh_demo -- mac

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;
use tokio::net::UdpSocket;

// ── config ──────────────────────────────────────────────────────────────────

const PORT: u16 = 4070;
const WIDTH: usize = 72;
const HEIGHT: usize = 22;
const FRAME_MS: u64 = 66; // ~15 fps
const HELLO_MS: u64 = 1000; // peer announcement interval
const PEER_TIMEOUT_MS: u64 = 5000; // peer considered dead after this

// Block characters for the plasma (dark → bright → dark)
const SHADES: &[char] = &[
    ' ', ' ', '·', ':', '░', '▒', '▓', '█', '▓', '▒', '░', ':', '·',
];

// ── peer state ──────────────────────────────────────────────────────────────

struct PeerInfo {
    _name: String,
    addr: SocketAddr,
    last_seen: Instant,
}

struct State {
    name: String,
    peers: BTreeMap<String, PeerInfo>,
    events_sent: u64,
    events_recv: u64,
    bytes_sent: u64,
    bytes_recv: u64,
}

// ── plasma renderer ─────────────────────────────────────────────────────────

fn epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Map a hue (0-360) to an ANSI 256-color code in the cyan/magenta range.
fn hue_to_color(hue: u32) -> u8 {
    // Use the 216-color cube (codes 16-231).
    // Map hue through a cyberpunk palette: blue → cyan → magenta → blue.
    let h = (hue % 360) as f64 / 360.0;
    let r = ((h * std::f64::consts::TAU + 2.0).sin() * 0.5 + 0.5) * 5.0;
    let g = ((h * std::f64::consts::TAU + 4.0).sin() * 0.3 + 0.2) * 5.0;
    let b = ((h * std::f64::consts::TAU).sin() * 0.5 + 0.5) * 5.0;
    let ri = (r as u8).min(5);
    let gi = (g as u8).min(5);
    let bi = (b as u8).min(5);
    16 + 36 * ri + 6 * gi + bi
}

fn render_frame(frame_num: u64) -> String {
    let t = frame_num as f64 * 0.08;
    let mut buf = String::with_capacity(WIDTH * HEIGHT * 16);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let fx = x as f64 / WIDTH as f64;
            let fy = y as f64 / HEIGHT as f64;

            // Layered sine plasma.
            let v1 = (fx * 10.0 + t * 1.2).sin();
            let v2 = (fy * 8.0 + t * 0.9).sin();
            let v3 = ((fx + fy) * 6.0 + t * 0.7).sin();
            let v4 = ((fx * fx + fy * fy).sqrt() * 12.0 - t * 1.5).sin();
            let v = v1 + v2 + v3 + v4;

            // Map to character.
            let norm = (v + 4.0) / 8.0; // 0..1
            let idx = (norm * (SHADES.len() - 1) as f64) as usize;
            let idx = idx.min(SHADES.len() - 1);

            // Map to color.
            let hue = ((norm * 360.0) as u32 + (frame_num as u32 * 3)) % 360;
            let color = hue_to_color(hue);

            buf.push_str(&format!("\x1b[38;5;{}m{}", color, SHADES[idx]));
        }
        buf.push_str("\x1b[0m\n");
    }
    buf
}

fn render_header(state: &State, frame_num: u64) -> String {
    let mut s = String::new();
    s.push_str("\x1b[1;36m"); // bold cyan

    // Top border
    s.push_str("╔══════════════════════════════════════════════════════════════════════╗\n");
    s.push_str(&format!(
        "║  NET MESH {:>60}║\n",
        format!(
            "node: {}  peers: {}  frame: {}",
            state.name,
            state.peers.len(),
            frame_num
        )
    ));
    s.push_str("╠══════════════════════════════════════════════════════════════════════╣\n");

    // Peer list
    if state.peers.is_empty() {
        s.push_str("║  waiting for peers...                                                ║\n");
    } else {
        for (name, peer) in &state.peers {
            let age_ms = peer.last_seen.elapsed().as_millis();
            let status = if age_ms < 2000 { "●" } else { "○" };
            let line = format!("  {} {}  ({})", status, name, peer.addr.ip());
            s.push_str(&format!("║{:<72}║\n", line));
        }
    }

    s.push_str("╠══════════════════════════════════════════════════════════════════════╣\n");
    s.push_str("\x1b[0m");
    s
}

fn render_footer(state: &State) -> String {
    let mut s = String::new();
    s.push_str("\x1b[1;36m");
    s.push_str("╠══════════════════════════════════════════════════════════════════════╣\n");
    let stats = format!(
        "  sent: {} events ({} B)   recv: {} events ({} B)",
        state.events_sent, state.bytes_sent, state.events_recv, state.bytes_recv,
    );
    s.push_str(&format!("║{:<72}║\n", stats));
    s.push_str("║  Ctrl+C to exit                                                      ║\n");
    s.push_str("╚══════════════════════════════════════════════════════════════════════╝\n");
    s.push_str("\x1b[0m");
    s
}

// ── network ─────────────────────────────────────────────────────────────────

async fn send_hello(sock: &UdpSocket, name: &str, broadcast: SocketAddr, state: &Mutex<State>) {
    let msg = format!("HELLO|{}", name);
    let _ = sock.send_to(msg.as_bytes(), broadcast).await;
    let mut s = state.lock();
    s.events_sent += 1;
    s.bytes_sent += msg.len() as u64;
}

async fn recv_loop(
    sock: Arc<UdpSocket>,
    state: Arc<Mutex<State>>,
    running: Arc<AtomicBool>,
    events_recv: Arc<AtomicU64>,
) {
    let mut buf = [0u8; 2048];
    while running.load(Ordering::Relaxed) {
        let result =
            tokio::time::timeout(Duration::from_millis(100), sock.recv_from(&mut buf)).await;
        if let Ok(Ok((len, addr))) = result {
            let msg = String::from_utf8_lossy(&buf[..len]);
            if let Some(name) = msg.strip_prefix("HELLO|") {
                let name = name.trim().to_string();
                let mut s = state.lock();
                if name != s.name {
                    s.peers.insert(
                        name.clone(),
                        PeerInfo {
                            _name: name,
                            addr,
                            last_seen: Instant::now(),
                        },
                    );
                    s.events_recv += 1;
                    s.bytes_recv += len as u64;
                }
            }
            events_recv.fetch_add(1, Ordering::Relaxed);
        }
    }
}

// ── main ────────────────────────────────────────────────────────────────────

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Parse name from CLI.
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| hostname_or_default());

    // Enable ANSI on Windows.
    #[cfg(windows)]
    let _ = enable_ansi_support();

    println!("\x1b[2J\x1b[H"); // clear screen
    println!(
        "\x1b[1;36mNET MESH DEMO — starting as '{}' on port {}...\x1b[0m\n",
        name, PORT
    );

    // Bind UDP socket.
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", PORT))
        .await
        .expect("failed to bind UDP socket — is another instance running?");
    sock.set_broadcast(true)
        .expect("failed to enable broadcast");

    let sock = Arc::new(sock);
    let broadcast: SocketAddr = format!("255.255.255.255:{}", PORT).parse().unwrap();

    let state = Arc::new(Mutex::new(State {
        name: name.clone(),
        peers: BTreeMap::new(),
        events_sent: 0,
        events_recv: 0,
        bytes_sent: 0,
        bytes_recv: 0,
    }));

    let running = Arc::new(AtomicBool::new(true));
    let events_recv = Arc::new(AtomicU64::new(0));

    // Spawn receiver.
    let recv_handle = tokio::spawn(recv_loop(
        sock.clone(),
        state.clone(),
        running.clone(),
        events_recv.clone(),
    ));

    // Send initial hello burst.
    for _ in 0..3 {
        send_hello(&sock, &name, broadcast, &state).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Main render loop.
    let start = Instant::now();
    let mut last_hello = Instant::now();
    let mut frame_count = 0u64;

    // Handle Ctrl+C.
    let running_ctrlc = running.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        running_ctrlc.store(false, Ordering::Relaxed);
    });

    while running.load(Ordering::Relaxed) {
        // Compute frame number from wall clock (all nodes derive the same number).
        let frame_num = epoch_ms() / FRAME_MS;

        // Prune dead peers.
        {
            let mut s = state.lock();
            s.peers
                .retain(|_, p| p.last_seen.elapsed().as_millis() < PEER_TIMEOUT_MS as u128);
        }

        // Periodic hello.
        if last_hello.elapsed().as_millis() >= HELLO_MS as u128 {
            send_hello(&sock, &name, broadcast, &state).await;
            last_hello = Instant::now();
        }

        // Render.
        let header = {
            let s = state.lock();
            render_header(&s, frame_num)
        };
        let plasma = render_frame(frame_num);
        let footer = {
            let s = state.lock();
            render_footer(&s)
        };

        // Move cursor to top-left and draw (no flicker).
        print!("\x1b[H{}{}{}", header, plasma, footer);

        frame_count += 1;
        tokio::time::sleep(Duration::from_millis(FRAME_MS)).await;
    }

    // Shutdown.
    running.store(false, Ordering::Relaxed);
    let _ = recv_handle.await;

    let elapsed = start.elapsed();
    let fps = frame_count as f64 / elapsed.as_secs_f64();

    println!("\x1b[2J\x1b[H");
    println!("\x1b[1;36mNET MESH DEMO — shutdown\x1b[0m\n");
    println!("  ran for:    {:.1}s", elapsed.as_secs_f64());
    println!("  frames:     {}", frame_count);
    println!("  avg fps:    {:.1}", fps);
    {
        let s = state.lock();
        println!("  sent:       {} events", s.events_sent);
        println!("  received:   {} events", s.events_recv);
        println!("  peers seen: {}", s.peers.len());
    }
}

fn hostname_or_default() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "node".into())
}

#[cfg(windows)]
fn enable_ansi_support() -> Result<(), ()> {
    use std::os::windows::io::AsRawHandle;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    unsafe {
        let handle = std::io::stdout().as_raw_handle();
        let mut mode: u32 = 0;
        if GetConsoleMode(handle as *mut _, &mut mode) == 0 {
            return Err(());
        }
        if SetConsoleMode(handle as *mut _, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) == 0 {
            return Err(());
        }
    }
    Ok(())
}

#[cfg(windows)]
extern "system" {
    fn GetConsoleMode(handle: *mut std::ffi::c_void, mode: *mut u32) -> i32;
    fn SetConsoleMode(handle: *mut std::ffi::c_void, mode: u32) -> i32;
}
