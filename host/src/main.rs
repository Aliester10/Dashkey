//! DashKey Host — binary legacy (GUI egui/eframe) + mode headless/pairing.
//!
//! Semua logika core ada di library `dashkey_host`; binary ini hanya
//! memilih mode jalannya: `pair`, `--no-gui`, atau GUI egui (default).

use std::sync::Arc;

use tracing::info;
use tracing_subscriber::EnvFilter;

use dashkey_host::{bind_server, data_dir, init_app, DEFAULT_PORT};

/// Cetak QR code pairing (berisi JSON: host, port, token) di terminal.
fn print_pair_qr(host_ip: &str, port: u16, token: &str) {
    let payload = serde_json::json!({
        "host": host_ip,
        "port": port,
        "token": token,
    })
    .to_string();

    use qrcode::QrCode;
    let code = match QrCode::new(&payload) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("gagal generate QR: {e}");
            println!("Pair payload (scan manual): {payload}");
            return;
        }
    };
    let rendered = code
        .render::<qrcode::render::unicode::Dense1x2>()
        .dark_color(qrcode::render::unicode::Dense1x2::Dark)
        .light_color(qrcode::render::unicode::Dense1x2::Light)
        .build();
    println!("Scan QR berikut dari aplikasi DashKey Controller:");
    println!("{rendered}");
    println!("Payload: {payload}");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("dashkey_host=info")),
        )
        .init();

    info!(version = env!("CARGO_PKG_VERSION"), "DashKey Host dimulai");

    let args: Vec<String> = std::env::args().collect();
    let port = std::env::var("DASHKEY_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    if args.len() > 1 && args[1] == "pair" {
        // Mode pairing: generate token, tampilkan QR, dan jalankan server
        // sementara (agar Controller bisa langsung scan + pair).
        let state = init_app(&data_dir(), true).await?;
        let token = state.pairing.generate_token();
        print_pair_qr(&state.host_ip, port, &token);

        let server = bind_server(port, Arc::clone(&state)).await?;
        info!("Mode pairing: server aktif di port {port}, token berlaku ±2 menit");
        let server_task = tokio::spawn(async move {
            let _ = server.run().await;
        });

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = tokio::time::sleep(std::time::Duration::from_secs(120)) => {
                info!("Pair token kedaluwarsa, mode pairing berhenti");
            }
        }
        server_task.abort();
        return Ok(());
    }

    let no_gui =
        args.iter().any(|a| a == "--no-gui") || std::env::var_os("DASHKEY_NO_GUI").is_some();

    let state = init_app(&data_dir(), true).await?;
    info!(host_name = %state.host_name, host_ip = %state.host_ip, "state siap");

    let server = bind_server(port, Arc::clone(&state)).await?;
    info!("Menunggu koneksi dari Controller di port {port}");

    if no_gui {
        info!("mode headless (--no-gui)");
        server.run().await?;
        return Ok(());
    }

    #[cfg(feature = "gui-egui")]
    {
        // Server jalan di background task; GUI di thread utama.
        let server_task = tokio::spawn({
            let server = Arc::clone(&server);
            async move {
                let _ = server.run().await;
            }
        });

        let gui_result = dashkey_host::gui::run(Arc::clone(&state), Arc::clone(&server), port);

        // GUI ditutup → hentikan server & keluar.
        server_task.abort();
        gui_result?;
    }

    #[cfg(not(feature = "gui-egui"))]
    {
        // Tanpa feature gui-egui (mis. dipakai binary Tauri), mode ini tak
        // dijalankan; cukup jalankan server headless.
        info!("mode headless (tanpa feature gui-egui)");
        server.run().await?;
    }

    Ok(())
}
