# Panduan Implementasi Desain DashKey ke `egui`

Poin penting yang harus dipahami dulu: mockup yang saya buat itu **HTML/CSS**, sedangkan project kamu pakai **`egui`** (immediate-mode GUI di Rust). Keduanya render dengan cara yang sangat berbeda — tidak bisa "copy-paste" langsung. Yang bisa dipindahkan adalah **bahasa desainnya**: palet warna, radius, spacing, tipografi, dan pola komponen (card, pill, icon chip). Panduan ini menerjemahkan itu semua jadi kode `egui` yang bisa langsung kamu pakai.

---

## 1. Setup Awal — Dependency Tambahan

Tambahkan ke `Cargo.toml`:

```toml
[dependencies]
eframe = "0.28"
egui = "0.28"
egui-phosphor = "0.6"   # icon font pengganti Tabler icons
```

`egui-phosphor` menyediakan ratusan ikon outline yang bisa dipakai persis seperti Tabler icons di mockup (mic, device, plug, dsb).

Daftarkan fontnya sekali di awal (biasanya di `main.rs` saat setup `eframe::run_native`):

```rust
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}
```

---

## 2. Definisikan Palet Warna Sebagai Konstanta

Bikin file `src/theme.rs` — ini jadi "single source of truth" warna, biar konsisten di semua halaman:

```rust
use egui::Color32;

pub struct Palette;

impl Palette {
    // surfaces
    pub const SURFACE_0: Color32 = Color32::from_rgb(0x1a, 0x1a, 0x1a);
    pub const SURFACE_1: Color32 = Color32::from_rgb(0x24, 0x24, 0x24);
    pub const SURFACE_2: Color32 = Color32::from_rgb(0x2c, 0x2c, 0x2c);

    // text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xf5, 0xf5, 0xf5);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0xa0, 0xa0, 0xa0);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x70, 0x70, 0x70);

    // accent (ungu — dipakai untuk tab aktif, brand)
    pub const ACCENT: Color32 = Color32::from_rgb(0x53, 0x4A, 0xB7);
    pub const ACCENT_TEXT_ON: Color32 = Color32::from_rgb(0xEE, 0xED, 0xFE);

    // role colors (untuk icon chip / status)
    pub const SUCCESS_BG: Color32 = Color32::from_rgb(0x0F, 0x3D, 0x30);
    pub const SUCCESS_TEXT: Color32 = Color32::from_rgb(0x5D, 0xCA, 0xA5);

    pub const BLUE_BG: Color32 = Color32::from_rgb(0x0C, 0x2A, 0x40);
    pub const BLUE_TEXT: Color32 = Color32::from_rgb(0x85, 0xB7, 0xEB);

    pub const PURPLE_BG: Color32 = Color32::from_rgb(0x26, 0x21, 0x5C);
    pub const PURPLE_TEXT: Color32 = Color32::from_rgb(0xAF, 0xA9, 0xEC);

    pub const AMBER_BG: Color32 = Color32::from_rgb(0x41, 0x24, 0x02);
    pub const AMBER_TEXT: Color32 = Color32::from_rgb(0xEF, 0x9F, 0x27);

    pub const CORAL_BG: Color32 = Color32::from_rgb(0x4A, 0x1B, 0x0C);
    pub const CORAL_TEXT: Color32 = Color32::from_rgb(0xF0, 0x99, 0x7B);
}

pub const RADIUS_CARD: f32 = 14.0;
pub const RADIUS_PILL: f32 = 20.0;
```

Kalau kamu mau support light mode juga, bikin dua struct (`PaletteDark`, `PaletteLight`) dan pilih berdasarkan `ctx.style().visuals.dark_mode`.

---

## 3. Komponen Reusable

Ini bagian paling penting — bikin fungsi helper sekali, dipakai berkali-kali di semua halaman.

### 3.1 Card dengan rounded corner

```rust
pub fn card(ui: &mut egui::Ui, fill: egui::Color32, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(fill)
        .rounding(RADIUS_CARD)
        .inner_margin(egui::Margin::same(16.0))
        .show(ui, add_contents);
}
```

Pemakaian:
```rust
card(ui, Palette::SURFACE_1, |ui| {
    ui.label("Isi card di sini");
});
```

### 3.2 Pill badge (untuk tab, status "online", dsb)

```rust
pub fn pill(ui: &mut egui::Ui, text: &str, bg: egui::Color32, fg: egui::Color32) {
    egui::Frame::none()
        .fill(bg)
        .rounding(RADIUS_PILL)
        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
        .show(ui, |ui| {
            ui.colored_label(fg, text);
        });
}
```

### 3.3 Icon chip bulat/rounded (dipakai di stat card & activity feed)

```rust
pub fn icon_chip(ui: &mut egui::Ui, icon: &str, bg: egui::Color32, fg: egui::Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    ui.painter().rect_filled(rect, size * 0.3, bg);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(size * 0.45),
        fg,
    );
}
```

Pemakaian (icon dari `egui-phosphor`):
```rust
icon_chip(ui, egui_phosphor::regular::PLUG, Palette::SUCCESS_BG, Palette::SUCCESS_TEXT, 30.0);
```

### 3.4 Stat card (gabungan icon chip + angka besar)

```rust
pub fn stat_card(ui: &mut egui::Ui, icon: &str, label: &str, value: &str, bg: Color32, fg: Color32) {
    card(ui, Palette::SURFACE_1, |ui| {
        ui.vertical(|ui| {
            icon_chip(ui, icon, bg, fg, 30.0);
            ui.add_space(10.0);
            ui.colored_label(Palette::TEXT_MUTED, label);
            ui.add_space(2.0);
            ui.label(egui::RichText::new(value).size(28.0).color(Palette::TEXT_PRIMARY));
        });
    });
}
```

---

## 4. Tab Bar Kustom (Pengganti Tab Bawaan)

Tab bar bawaan `egui` (underline biasa) tidak akan terasa seperti pill di mockup. Bikin manual:

```rust
pub fn tab_button(ui: &mut egui::Ui, icon: &str, label: &str, active: bool) -> bool {
    let (bg, fg) = if active {
        (Palette::ACCENT, Palette::ACCENT_TEXT_ON)
    } else {
        (egui::Color32::TRANSPARENT, Palette::TEXT_SECONDARY)
    };

    let response = egui::Frame::none()
        .fill(bg)
        .rounding(RADIUS_PILL)
        .inner_margin(egui::Margin::symmetric(12.0, 7.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(fg, icon);
                ui.colored_label(fg, label);
            });
        })
        .response;

    ui.interact(response.rect, response.id, egui::Sense::click()).clicked()
}
```

Pemakaian di `TopBottomPanel`:
```rust
egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
    ui.horizontal(|ui| {
        if tab_button(ui, egui_phosphor::regular::SQUARES_FOUR, "Dashboard", self.active_tab == Tab::Dashboard) {
            self.active_tab = Tab::Dashboard;
        }
        if tab_button(ui, egui_phosphor::regular::GRID_FOUR, "Buttons", self.active_tab == Tab::Buttons) {
            self.active_tab = Tab::Buttons;
        }
        // ...tab lainnya
    });
});
```

---

## 5. Susun Halaman Dashboard

Setelah semua komponen di atas ada, halaman Dashboard tinggal dirangkai:

```rust
egui::CentralPanel::default()
    .frame(egui::Frame::none().fill(Palette::SURFACE_0).inner_margin(20.0))
    .show(ctx, |ui| {
        // Hero banner
        card(ui, Palette::SURFACE_1, |ui| {
            ui.horizontal(|ui| {
                icon_chip(ui, egui_phosphor::regular::LIGHTNING, Palette::ACCENT, Palette::ACCENT_TEXT_ON, 44.0);
                ui.vertical(|ui| {
                    ui.heading("Selamat datang di DashKey");
                    ui.colored_label(Palette::TEXT_MUTED, "Command center untuk mengelola tombol, device, dan integrasi PC.");
                });
            });
        });

        ui.add_space(12.0);

        // Grid 4 stat card
        ui.columns(4, |columns| {
            stat_card(&mut columns[0], egui_phosphor::regular::PLUG, "DEVICE ONLINE", "1", Palette::SUCCESS_BG, Palette::SUCCESS_TEXT);
            stat_card(&mut columns[1], egui_phosphor::regular::USER_CIRCLE, "PROFILE", "2", Palette::BLUE_BG, Palette::BLUE_TEXT);
            stat_card(&mut columns[2], egui_phosphor::regular::STACK, "PAGE", "4", Palette::PURPLE_BG, Palette::PURPLE_TEXT);
            stat_card(&mut columns[3], egui_phosphor::regular::SQUARE, "BUTTON", "17", Palette::AMBER_BG, Palette::AMBER_TEXT);
        });
    });
```

---

## 6. Tips Praktis Supaya Hasilnya Konsisten

1. **Jangan hardcode warna di tiap halaman** — selalu panggil dari `Palette::...`. Kalau nanti mau ganti tone warna, cukup edit satu file.
2. **`ui.columns()` untuk grid rapi** — dipakai di stat card dan grid tombol Elgato-style nanti.
3. **Icon konsisten** — pastikan pakai satu icon set saja (`egui-phosphor`), jangan campur dengan icon custom lain supaya "berat visual" tiap ikon sama.
4. **Radius konsisten** — pakai konstanta `RADIUS_CARD` (14px) untuk semua card, `RADIUS_PILL` (20px) untuk semua badge/tab. Ini yang bikin desain "terasa satu keluarga" walau beda halaman.
5. **Test di light mode juga** — kalau target Windows, banyak user pakai light mode OS. Siapkan varian warna terang biar tidak pecah.
6. **Commit `theme.rs` dan `widgets.rs` (helper di atas) sebagai modul terpisah** — supaya halaman Buttons, Profiles, Pairing nanti tinggal import dan pakai ulang, bukan tulis ulang styling tiap halaman.

---

## 7. Urutan Kerja yang Disarankan

1. Bikin `theme.rs` (palet warna) — 15 menit.
2. Bikin `widgets.rs` (card, pill, icon_chip, stat_card, tab_button) — 30–45 menit.
3. Setup font `egui-phosphor` di `main.rs`.
4. Refactor halaman Dashboard yang sudah ada, ganti komponen lama satu-satu pakai helper baru.
5. Setelah Dashboard konsisten, halaman lain (Buttons, Pairing, Devices) tinggal reuse komponen yang sama — jauh lebih cepat karena fondasi sudah ada.





# Panduan Implementasi Desain Halaman Buttons ke `egui`

Lanjutan dari `EGUI_DESIGN_IMPLEMENTATION.md` sebelumnya. Panduan ini fokus ke komponen baru yang muncul di halaman **Buttons**: list tombol dengan state selected, color swatch picker, icon picker, action chain editor, dan preview card. Semua tetap pakai `Palette` dan helper (`card`, `pill`, `icon_chip`) dari file sebelumnya — di sini kita nambah yang belum ada.

---

## 1. Struktur Data

Sebelum ke UI, siapkan dulu model data yang dipakai halaman ini (sudah sejalan dengan skema Button di PRD):

```rust
#[derive(Clone, PartialEq)]
pub struct ButtonAction {
    pub kind: ActionKind,
    pub description: String, // ditampilkan di action chain, mis. "Media Play/Pause"
}

#[derive(Clone, PartialEq)]
pub enum ActionKind {
    KeyboardShortcut,
    OpenApp,
    RunCommand,
    OpenUrl,
    PlaySound,
    MediaControl,
    ObsAction,
}

impl ActionKind {
    pub fn icon(&self) -> &'static str {
        match self {
            ActionKind::KeyboardShortcut => egui_phosphor::regular::KEYBOARD,
            ActionKind::OpenApp => egui_phosphor::regular::APP_WINDOW,
            ActionKind::RunCommand => egui_phosphor::regular::TERMINAL,
            ActionKind::OpenUrl => egui_phosphor::regular::LINK,
            ActionKind::PlaySound => egui_phosphor::regular::SPEAKER_HIGH,
            ActionKind::MediaControl => egui_phosphor::regular::PLAY,
            ActionKind::ObsAction => egui_phosphor::regular::BROADCAST,
        }
    }
}

#[derive(Clone)]
pub struct ButtonConfig {
    pub id: String,
    pub label: String,
    pub color: egui::Color32,
    pub icon: &'static str,
    pub actions: Vec<ButtonAction>,
}
```

State halaman:

```rust
pub struct ButtonsPage {
    pub buttons: Vec<ButtonConfig>,
    pub selected_index: Option<usize>,
}
```

---

## 2. Komponen Baru

### 2.1 Row tombol di list kiri (dengan state selected)

```rust
pub fn button_row(ui: &mut egui::Ui, label: &str, color: egui::Color32, selected: bool) -> egui::Response {
    let bg = if selected { Palette::SURFACE_2 } else { egui::Color32::TRANSPARENT };

    let frame = egui::Frame::none()
        .fill(bg)
        .rounding(10.0)
        .inner_margin(egui::Margin::symmetric(8.0, 8.0));

    let response = frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            ui.painter().circle_filled(dot_rect.center(), 4.0, color);
            ui.label(label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let _ = ui.small_button(egui_phosphor::regular::TRASH);
                let _ = ui.small_button(egui_phosphor::regular::DOTS_SIX_VERTICAL);
            });
        });
    }).response;

    // bikin seluruh row clickable, bukan cuma teksnya
    ui.interact(response.rect, response.id.with("row_click"), egui::Sense::click())
}
```

Pemakaian di `SidePanel`:
```rust
for (i, btn) in self.buttons.iter().enumerate() {
    let selected = self.selected_index == Some(i);
    if button_row(ui, &btn.label, btn.color, selected).clicked() {
        self.selected_index = Some(i);
    }
}
```

### 2.2 Color swatch picker

```rust
pub fn color_swatch_picker(ui: &mut egui::Ui, options: &[egui::Color32], selected: &mut egui::Color32) {
    ui.horizontal_wrapped(|ui| {
        for &c in options {
            let is_selected = *selected == c;
            let (rect, response) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::click());
            ui.painter().rect_filled(rect, 8.0, c);
            if is_selected {
                ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(2.0, Palette::TEXT_PRIMARY));
            }
            if response.clicked() {
                *selected = c;
            }
        }
    });
}
```

Palet warna yang disarankan (dari mockup), taruh di `theme.rs`:
```rust
pub const BUTTON_COLOR_OPTIONS: &[egui::Color32] = &[
    egui::Color32::from_rgb(0xAF, 0x9E, 0xEC), // purple
    egui::Color32::from_rgb(0x5D, 0xCA, 0xA5), // teal
    egui::Color32::from_rgb(0xF0, 0x99, 0x7B), // coral
    egui::Color32::from_rgb(0xED, 0x93, 0xB1), // pink
    egui::Color32::from_rgb(0xEF, 0x9F, 0x27), // amber
    egui::Color32::from_rgb(0xE2, 0x4B, 0x4A), // red
];
```

### 2.3 Icon picker

```rust
pub fn icon_picker(ui: &mut egui::Ui, options: &[&'static str], selected: &mut &'static str) {
    ui.horizontal_wrapped(|ui| {
        for &icon in options {
            let is_selected = *selected == icon;
            let bg = Palette::SURFACE_2;
            let (rect, response) = ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::click());
            ui.painter().rect_filled(rect, 9.0, bg);
            if is_selected {
                ui.painter().rect_stroke(rect, 9.0, egui::Stroke::new(1.5, Palette::ACCENT));
            }
            let fg = if is_selected { Palette::TEXT_PRIMARY } else { Palette::TEXT_SECONDARY };
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, icon, egui::FontId::proportional(16.0), fg);
            if response.clicked() {
                *selected = icon;
            }
        }
    });
}
```

### 2.4 Action chain list

```rust
pub fn action_chain(ui: &mut egui::Ui, actions: &mut Vec<ButtonAction>) {
    let mut remove_index: Option<usize> = None;

    for (i, action) in actions.iter().enumerate() {
        card(ui, Palette::SURFACE_2, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(Palette::TEXT_MUTED, format!("{}", i + 1));
                ui.colored_label(Palette::PURPLE_TEXT, action.kind.icon());
                ui.label(&action.description);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(egui_phosphor::regular::X).clicked() {
                        remove_index = Some(i);
                    }
                });
            });
        });
        ui.add_space(4.0);
    }

    if let Some(i) = remove_index {
        actions.remove(i);
    }
}
```

> Catatan: menambah aksi baru sebaiknya buka `egui::Window` atau modal kecil berisi pilihan jenis aksi (`ActionKind`) + field target-nya (misal path aplikasi, isi command, URL). Modal ini beda-beda tergantung `ActionKind`, jadi dirender pakai `match` per varian.

### 2.5 Preview card (live preview tombol)

```rust
pub fn button_preview(ui: &mut egui::Ui, label: &str, icon: &'static str, color: egui::Color32) {
    let size = egui::vec2(160.0, 160.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect_filled(rect, 14.0, color);

    // pilih warna teks gelap/terang otomatis berdasar brightness bg
    let brightness = (color.r() as u32 + color.g() as u32 + color.b() as u32) / 3;
    let text_color = if brightness > 140 { egui::Color32::from_rgb(0x26, 0x21, 0x5C) } else { egui::Color32::WHITE };

    let icon_pos = rect.center() - egui::vec2(0.0, 14.0);
    ui.painter().text(icon_pos, egui::Align2::CENTER_CENTER, icon, egui::FontId::proportional(26.0), text_color);

    let label_pos = rect.center() + egui::vec2(0.0, 18.0);
    ui.painter().text(label_pos, egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(12.0), text_color);
}
```

Fungsi kecil di atas (`brightness` check) penting supaya preview tetap kebaca kalau user pilih warna terang (misal kuning) — teks otomatis jadi gelap, bukan putih yang nge-blend.

---

## 3. Merangkai Halaman Buttons

```rust
egui::SidePanel::left("buttons_sidebar")
    .exact_width(260.0)
    .frame(egui::Frame::none().fill(Palette::SURFACE_1).inner_margin(16.0))
    .show(ctx, |ui| {
        ui.label(egui::RichText::new("Pages & tombol").size(14.0).strong());
        ui.add_space(10.0);

        for (i, btn) in page.buttons.iter().enumerate() {
            let selected = page.selected_index == Some(i);
            if button_row(ui, &btn.label, btn.color, selected).clicked() {
                page.selected_index = Some(i);
            }
        }

        ui.add_space(12.0);
        if ui.button(format!("{} Tambah tombol", egui_phosphor::regular::PLUS)).clicked() {
            // push ButtonConfig baru, auto-select
        }
    });

egui::CentralPanel::default()
    .frame(egui::Frame::none().fill(Palette::SURFACE_0).inner_margin(20.0))
    .show(ctx, |ui| {
        if let Some(i) = page.selected_index {
            let btn = &mut page.buttons[i];

            ui.columns(2, |columns| {
                // kolom kiri: form editor
                card(&mut columns[0], Palette::SURFACE_1, |ui| {
                    ui.colored_label(Palette::TEXT_MUTED, "LABEL");
                    ui.text_edit_singleline(&mut btn.label);

                    ui.add_space(10.0);
                    ui.colored_label(Palette::TEXT_MUTED, "WARNA");
                    color_swatch_picker(ui, BUTTON_COLOR_OPTIONS, &mut btn.color);

                    ui.add_space(10.0);
                    ui.colored_label(Palette::TEXT_MUTED, "IKON");
                    icon_picker(ui, &[
                        egui_phosphor::regular::PLAY,
                        egui_phosphor::regular::PAUSE,
                        egui_phosphor::regular::SKIP_FORWARD,
                        egui_phosphor::regular::SPEAKER_HIGH,
                    ], &mut btn.icon);
                });

                // kolom kanan: preview
                columns[1].vertical_centered(|ui| {
                    button_preview(ui, &btn.label, btn.icon, btn.color);
                });
            });

            ui.add_space(12.0);
            ui.colored_label(Palette::TEXT_MUTED, "ACTION CHAIN");
            action_chain(ui, &mut btn.actions);
        } else {
            ui.colored_label(Palette::TEXT_MUTED, "Pilih tombol di kiri, atau tambah tombol baru.");
        }
    });
```

---

## 4. Urutan Kerja Disarankan

1. Tambah struct `ButtonConfig`, `ButtonAction`, `ActionKind` ke module data kamu.
2. Tambah 5 fungsi komponen baru (`button_row`, `color_swatch_picker`, `icon_picker`, `action_chain`, `button_preview`) ke `widgets.rs`.
3. Refactor halaman Buttons yang sekarang jadi 2 panel (`SidePanel` + `CentralPanel`) sesuai contoh di atas.
4. Setelah layout dasar jalan, baru bikin modal "Tambah aksi" (butuh sedikit effort lebih karena field-nya beda tiap `ActionKind`).
5. Terakhir, sambungkan `ButtonConfig` ini ke `Config Store` (JSON/SQLite di sisi Host) supaya perubahan di UI benar-benar tersimpan dan ke-broadcast ke Controller.