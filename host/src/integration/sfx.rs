//! SFX Importer — impor suara dari www.myinstants.com.
//!
//! Input yang diterima (dari Controller):
//! 1. Kode embed iframe (diambil `src="..."`).
//! 2. URL halaman: https://www.myinstants.com/instant/<slug>-<id>/
//! 3. URL embed:  https://www.myinstants.com/instant/<slug>-<id>/embed/
//! 4. URL mp3 langsung: https://www.myinstants.com/media/sounds/<file>.mp3
//!
//! Alur: fetch halaman embed → ekstrak path `media/sounds/*.mp3` →
//! download ke `sounds/` → kembalikan nama file + nama tombol.

use std::path::Path;

use tracing::info;

/// Hasil impor SFX.
#[derive(Debug, Clone)]
pub struct SfxImport {
    /// Nama file yang tersimpan di `sounds/` (mis. `fahhhhhhhhhhhhhh.mp3`).
    pub file_name: String,
    /// Nama tombol yang disarankan (dari slug/title).
    pub button_name: String,
}

/// Normalisasi input pengguna (iframe HTML atau URL mentah) menjadi URL.
pub fn normalize_input(input: &str) -> String {
    let trimmed = input.trim();
    // Ambil src dari tag iframe bila ada.
    if let Some(start) = trimmed.find("src=") {
        let rest = &trimmed[start + 4..];
        let rest = rest.trim_start();
        let quote = rest.chars().next();
        let inner = if quote == Some('"') || quote == Some('\'') {
            let q = quote.unwrap();
            let end = rest[1..].find(q).map(|i| i + 1).unwrap_or(rest.len() - 1);
            &rest[1..end]
        } else {
            let end = rest
                .find(|c: char| c.is_whitespace() || c == '>')
                .unwrap_or(rest.len());
            &rest[..end]
        };
        return normalize_url(inner);
    }
    normalize_url(trimmed)
}

/// Normalisasi URL MyInstants.
/// - `/instant/<slug>/` atau `/instant/<slug>/embed/` → bentuk embed.
/// - `/media/sounds/<file>.mp3` → langsung.
fn normalize_url(url: &str) -> String {
    let mut url = url.to_string();
    if url.contains("/media/sounds/") {
        return url;
    }
    // Pastikan ada protokol.
    if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("https://{url}");
    }
    // Normalisasi /instant/<slug>-<id>/ menjadi /instant/<slug>-<id>/embed/.
    if url.contains("/instant/") {
        let trimmed = url.trim_end_matches('/');
        if !trimmed.ends_with("/embed") {
            url = format!("{trimmed}/embed/");
        } else {
            url = format!("{trimmed}/");
        }
    }
    url
}

/// Ekstrak path `media/sounds/*.mp3` dari HTML halaman embed.
pub fn extract_sound_path(html: &str) -> Option<String> {
    let needle = "/media/sounds/";
    let idx = html.find(needle)?;
    let rest = &html[idx..];
    let end = rest
        .find(|c: char| {
            matches!(
                c,
                '"' | '\'' | '<' | '>' | ' ' | '\\' | ')' | '(' | '\n' | '\r'
            )
        })
        .unwrap_or(rest.len());
    let path = &rest[..end];
    if path.ends_with(".mp3") || path.ends_with(".wav") {
        Some(path.to_string())
    } else {
        None
    }
}

/// Ekstrak nama tombol dari slug `/instant/<name>-<id>/`.
pub fn extract_button_name(url: &str) -> Option<String> {
    let marker = "/instant/";
    let idx = url.find(marker)?;
    let rest = &url[idx + marker.len()..];
    let end = rest.find('/').unwrap_or(rest.len());
    let slug = &rest[..end];
    // Hapus suffix "-<angka>" (id instan).
    let name = match slug.rfind('-') {
        Some(pos) if slug[pos + 1..].chars().all(|c| c.is_ascii_digit()) => &slug[..pos],
        _ => slug,
    };
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// Bersihkan nama file agar aman untuk filesystem.
fn sanitize_file_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Import SFX dari MyInstants ke folder [sounds_dir].
pub async fn import_sfx(input: &str, sounds_dir: &Path) -> Result<SfxImport, String> {
    let url = normalize_input(input);
    info!(%url, "import_sfx dimulai");

    // Kasus: URL mp3 langsung.
    if url.contains("/media/sounds/") {
        let file_name = url
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .ok_or("URL file tidak valid")?;
        let safe = sanitize_file_name(file_name);
        let dest = sounds_dir.join(&safe);
        download(&url, &dest).await?;
        let button_name = extract_button_name(&url)
            .or_else(|| file_name.strip_suffix(".mp3").map(|s| s.to_string()))
            .unwrap_or_else(|| "SFX".into());
        return Ok(SfxImport {
            file_name: safe,
            button_name,
        });
    }

    // Pastikan hanya myinstants.com.
    let host_ok = url.starts_with("https://www.myinstants.com/")
        || url.starts_with("http://www.myinstants.com/")
        || url.starts_with("https://myinstants.com/")
        || url.starts_with("http://myinstants.com/");
    if !host_ok {
        return Err("URL harus dari myinstants.com (halaman instan, embed, atau mp3)".into());
    }
    if !url.contains("/instant/") {
        return Err("URL bukan halaman instan myinstants (harus mengandung /instant/...)".into());
    }

    // Fetch halaman embed.
    let html = fetch_text(&url).await?;
    let path = extract_sound_path(&html).ok_or("tidak menemukan file suara di halaman embed")?;
    let mp3_url = format!("https://www.myinstants.com{path}");

    let file_name = path
        .rsplit('/')
        .next()
        .map(sanitize_file_name)
        .ok_or("nama file tidak valid")?;
    let dest = sounds_dir.join(&file_name);

    // Skip download bila sudah ada (idempoten).
    if !dest.exists() {
        download(&mp3_url, &dest).await?;
    } else {
        info!(?dest, "file suara sudah ada, lewati download");
    }

    let button_name =
        extract_button_name(&url).unwrap_or_else(|| file_name.trim_end_matches(".mp3").to_string());

    Ok(SfxImport {
        file_name,
        button_name,
    })
}

async fn fetch_text(url: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) DashKey/0.1")
        .build()
        .map_err(|e| format!("gagal inisialisasi http client: {e}"))?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("gagal fetch {url}: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("myinstants menjawab HTTP {status}"));
    }
    resp.text()
        .await
        .map_err(|e| format!("gagal baca body: {e}"))
}

async fn download(url: &str, dest: &Path) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) DashKey/0.1")
        .build()
        .map_err(|e| format!("gagal inisialisasi http client: {e}"))?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("gagal download {url}: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("download menjawab HTTP {status}"));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("gagal baca body: {e}"))?;
    std::fs::write(dest, &bytes).map_err(|e| format!("gagal menyimpan file: {e}"))?;
    info!(?dest, size = bytes.len(), "sfx tersimpan");
    Ok(())
}

/// Muat file gambar lokal sebagai base64 (untuk sinkronisasi ikon ke HP).
/// Hanya PNG/JPG/JPEG/WEBP/ICO dengan ukuran maksimal 1 MB.
pub fn load_image_base64(path: &str) -> Option<String> {
    let p = Path::new(path);
    let ext = p.extension()?.to_str()?.to_ascii_lowercase();
    if !matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "ico" | "gif"
    ) {
        return None;
    }
    let meta = std::fs::metadata(p).ok()?;
    if meta.len() > 1024 * 1024 {
        return None;
    }
    let bytes = std::fs::read(p).ok()?;
    use base64::Engine;
    Some(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_iframe_html() {
        let iframe = r#"<iframe width="110" height="200" src="https://www.myinstants.com/instant/fahhhhhhhhhhhhhh-3525/embed/" frameborder="0" scrolling="no"></iframe>"#;
        assert_eq!(
            normalize_input(iframe),
            "https://www.myinstants.com/instant/fahhhhhhhhhhhhhh-3525/embed/"
        );
    }

    #[test]
    fn normalize_plain_url_adds_protocol() {
        assert_eq!(
            normalize_input("www.myinstants.com/instant/abc-123/embed/"),
            "https://www.myinstants.com/instant/abc-123/embed/"
        );
    }

    #[test]
    fn normalize_instant_page_to_embed() {
        assert_eq!(
            normalize_input("https://www.myinstants.com/instant/abc-123/"),
            "https://www.myinstants.com/instant/abc-123/embed/"
        );
    }

    #[test]
    fn normalize_direct_mp3_unchanged() {
        let url = "https://www.myinstants.com/media/sounds/abc.mp3";
        assert_eq!(normalize_input(url), url);
    }

    #[test]
    fn extract_sound_path_from_embed_html() {
        let html = r#"<div class="instant"><button onclick="play('/media/sounds/fahhhhhhhhhhhhhh.mp3')"></button><audio src="https://www.myinstants.com/media/sounds/other.mp3"></audio></div>"#;
        assert_eq!(
            extract_sound_path(html),
            Some("/media/sounds/fahhhhhhhhhhhhhh.mp3".into())
        );
    }

    #[test]
    fn extract_sound_path_none_when_missing() {
        assert_eq!(extract_sound_path("<html></html>"), None);
    }

    #[test]
    fn extract_button_name_strips_id() {
        assert_eq!(
            extract_button_name("https://www.myinstants.com/instant/fahhhhhhhhhhhhhh-3525/embed/"),
            Some("fahhhhhhhhhhhhhh".into())
        );
        assert_eq!(
            extract_button_name("https://www.myinstants.com/instant/bruh/"),
            Some("bruh".into())
        );
    }

    #[test]
    fn sanitize_keeps_safe_chars() {
        assert_eq!(sanitize_file_name("abc-123.mp3"), "abc-123.mp3");
        assert_eq!(sanitize_file_name("a b/c.mp3"), "a_b_c.mp3");
    }

    #[test]
    fn reject_non_myinstants_url() {
        let result = tokio::runtime::Runtime::new().unwrap().block_on(import_sfx(
            "https://example.com/instant/abc-123/",
            Path::new("/tmp"),
        ));
        assert!(result.is_err());
    }
}
