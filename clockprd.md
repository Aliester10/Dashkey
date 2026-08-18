# DashKey Flip Clock

> **DashKey Flip Clock** adalah halaman Clock Mode khusus pada aplikasi DashKey yang menampilkan waktu secara realtime menggunakan desain **mechanical flip clock** dengan gaya **Dark Neumorphism**.

---

## 1. Overview

DashKey membutuhkan halaman khusus yang berfungsi sebagai **fullscreen clock display**.

Halaman ini tidak menampilkan action button, sidebar, search bar, atau elemen aplikasi lainnya. Fokus utama halaman adalah menampilkan waktu secara besar, jelas, dan elegan.

Format waktu utama:

```text
10 : 47 : 56
```

Setiap digit yang berubah harus melakukan **flip animation** seperti mechanical flip clock.

Contoh:

```text
10 : 47 : 56
         ↓
10 : 47 : 57
```

Hanya digit `6 → 7` yang melakukan animasi.

---

# 2. Goals

Fitur ini memiliki tujuan:

* Menampilkan waktu sistem secara realtime.
* Menampilkan waktu dalam format `HH : MM : SS`.
* Memberikan efek mechanical flip pada setiap perubahan digit.
* Mendukung portrait dan landscape.
* Mendukung fullscreen.
* Menggunakan desain Dark Neumorphism.
* Responsive pada mobile, tablet, dan desktop.
* Memiliki performa animasi yang smooth.

---

# 3. Design Direction

### Style

**Dark Neumorphism + Mechanical Flip Clock**

Karakter visual:

* Dark.
* Minimal.
* Soft shadow.
* Rounded card.
* Subtle depth.
* High readability.
* Tidak terlalu banyak dekorasi.
* Clock menjadi fokus utama.

Referensi visual utama:

```text
┌──────────────┐  ┌──────────────┐
│              │  │              │
│      1       │  │      0       │
│              │  │              │
├──────────────┤  ├──────────────┤
│      1       │  │      0       │
│              │  │              │
└──────────────┘  └──────────────┘

         10 : 47 : 56
```

---

# 4. Supported Platforms

Target platform:

* Android
* iOS
* Windows
* macOS
* Linux

Framework:

```text
Flutter
```

Target orientation:

```text
Portrait
Landscape
```

---

# 5. User Flow

```text
DashKey
   │
   └── Clock
        │
        └── Clock Mode
             │
             ├── Current Time
             ├── Flip Animation
             ├── Date
             └── Fullscreen
```

Ketika user masuk ke Clock:

```text
Clock Page
    ↓
Get system time
    ↓
Display HH : MM : SS
    ↓
Update every second
    ↓
Detect changed digit
    ↓
Run flip animation
```

---

# 6. Portrait Layout

Pada portrait, clock berada di tengah layar.

```text
┌─────────────────────────────┐
│                             │
│                             │
│                             │
│      ┌─────┐  ┌─────┐      │
│      │  1  │  │  0  │      │
│      └─────┘  └─────┘      │
│                             │
│           :                 │
│                             │
│      ┌─────┐  ┌─────┐      │
│      │  4  │  │  7  │      │
│      └─────┘  └─────┘      │
│                             │
│           :                 │
│                             │
│      ┌─────┐  ┌─────┐      │
│      │  5  │  │  6  │      │
│      └─────┘  └─────┘      │
│                             │
│       TUESDAY, AUG 18       │
│                             │
└─────────────────────────────┘
```

Jika lebar device mencukupi, format dapat tetap dibuat satu baris:

```text
10 : 47 : 56
```

Layout harus responsive dan tidak boleh menyebabkan overflow.

---

# 7. Landscape Layout

Landscape menggunakan layar secara maksimal.

```text
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│                                                              │
│             ┌────┐ ┌────┐    ┌────┐ ┌────┐                 │
│             │ 10 │ │    │    │ 47 │ │    │                 │
│             └────┘ └────┘    └────┘ └────┘                 │
│                                                              │
│                    10 : 47 : 56                              │
│                                                              │
│                  AUG 18, 2026                                │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

Landscape harus:

* Menggunakan clock dengan ukuran lebih besar.
* Center horizontally.
* Center vertically.
* Tidak memiliki sidebar.
* Tidak memiliki bottom navigation.
* Tidak memiliki action cards.
* Tidak memiliki search bar.

---

# 8. Flip Clock Structure

Struktur utama:

```text
FlipClock
├── Hour
│   ├── FlipDigit
│   └── FlipDigit
│
├── Separator
│
├── Minute
│   ├── FlipDigit
│   └── FlipDigit
│
├── Separator
│
└── Second
    ├── FlipDigit
    └── FlipDigit
```

Format:

```text
HH : MM : SS
```

Contoh:

```text
10 : 47 : 56
```

---

# 9. Flip Digit

Setiap digit harus terdiri dari dua bagian:

```text
┌─────────────────┐
│                 │
│        5        │
│                 │
├─────────────────┤
│                 │
│        5        │
│                 │
└─────────────────┘
```

Structure:

```text
FlipDigit
├── TopHalf
├── Divider
└── BottomHalf
```

Digit tidak boleh hanya berupa satu `Text` widget dengan animasi scale.

Harus terdapat visual separation antara upper dan lower half agar menghasilkan efek flip clock yang realistis.

---

# 10. Flip Animation

Animation digunakan ketika nilai digit berubah.

Contoh:

```text
5 → 6
```

Animation sequence:

```text
Old Digit
    │
    ▼
Top flap rotates
    │
    ▼
Middle divider
    │
    ▼
Bottom flap reveals
    │
    ▼
New Digit
```

Recommended:

```text
Duration: 300–500ms
Curve: easeInOut
```

Animation harus terlihat seperti kartu mekanik yang terlipat, bukan sekadar fade atau scale.

---

# 11. Changed Digit Only

Hanya digit yang berubah yang boleh melakukan animation.

Contoh:

```text
10 : 47 : 56
         ↓
10 : 47 : 57
```

Yang melakukan animation:

```text
6 → 7
```

Yang tidak berubah:

```text
1
0
4
7
5
```

harus tetap statis.

---

# 12. Minute Transition

Ketika terjadi perubahan menit:

```text
10 : 47 : 59
```

menjadi:

```text
10 : 48 : 00
```

maka digit yang berubah melakukan animation secara bersamaan sesuai kebutuhan.

---

# 13. Time Source

Clock harus menggunakan waktu sistem/device.

Format default:

```text
HH:mm:ss
```

Gunakan:

```text
24-hour format
```

Contoh:

```text
22 : 48 : 37
```

12-hour format dapat ditambahkan sebagai fitur future.

---

# 14. Date Display

Tanggal ditampilkan di bawah clock.

Contoh:

```text
TUESDAY, AUG 18, 2026
```

Recommended style:

```text
Font Size: 14–20px
Font Weight: Medium
Letter Spacing: 1–2px
Opacity: 60–80%
```

Date harus tetap lebih kecil daripada clock.

---

# 15. Color Palette

## Background

```text
#0F1115
```

atau:

```text
#111111
```

## Card

```text
#303030
```

## Card Highlight

```text
#3A3A3A
```

## Primary Accent

```text
#6366F1
```

## Secondary Accent

```text
#8B5CF6
```

## Text Primary

```text
#E8E8E8
```

## Text Secondary

```text
#8B93A7
```

## Divider

```text
#1B1B1B
```

## Status / Success

```text
#22C55E
```

---

# 16. Neumorphism

Card menggunakan kombinasi raised dan inset shadow.

### Raised

```text
Light Shadow:
rgba(255,255,255,0.05)

Dark Shadow:
rgba(0,0,0,0.45)
```

### Pressed / Inset

```text
Inner Light:
rgba(255,255,255,0.03)

Inner Dark:
rgba(0,0,0,0.35)
```

Tujuannya menghasilkan efek:

```text
Dark Surface
      ↓
Soft Depth
      ↓
Raised Card
```

Hindari shadow yang terlalu kuat sehingga tampilan terlihat seperti glassmorphism.

---

# 17. Typography

Gunakan:

```text
Font Family: Inter
```

### Clock

```text
Weight: Bold
Size: Responsive
```

Recommended range:

```text
Portrait:
120–170px

Landscape:
180–240px
```

Ukuran sebenarnya harus dihitung berdasarkan available screen size.

### Date

```text
14–20px
Medium
```

---

# 18. Flip Card Size

## Portrait

Recommended:

```text
Width: 80–150px
Height: 120–220px
```

## Landscape

Recommended:

```text
Width: 150–220px
Height: 220–300px
```

Jangan hardcode ukuran berdasarkan satu device.

Gunakan responsive calculation berdasarkan:

```text
availableWidth
availableHeight
orientation
```

---

# 19. Border Radius

Flip card:

```text
18–24px
```

Small UI elements:

```text
12–16px
```

Divider:

```text
1–2px
```

---

# 20. Separator

Separator menggunakan:

```text
:
```

Contoh:

```text
10 : 47 : 56
```

Separator harus memiliki visual yang lebih subtle daripada digit.

Recommended:

```text
Color: #777777
```

---

# 21. Fullscreen

Clock Mode harus mendukung fullscreen.

Ketika fullscreen:

```text
Status Bar      → Hidden
Navigation Bar  → Hidden
App Controls    → Hidden
```

Hasil:

```text
┌────────────────────────────────────┐
│                                    │
│                                    │
│            10 : 47 : 56            │
│                                    │
│          AUG 18, 2026              │
│                                    │
└────────────────────────────────────┘
```

Clock menjadi fokus utama layar.

---

# 22. Interaction

Clock pada dasarnya bersifat passive.

### MVP

Tidak perlu banyak interaction.

Optional:

```text
Tap
↓
Toggle UI visibility
```

Future:

```text
Double Tap
↓
Exit Clock Mode
```

Future:

```text
Swipe Left / Right
↓
Change Clock Theme
```

Gesture bukan bagian dari MVP.

---

# 23. Responsive Behavior

### Portrait

```text
Orientation = Portrait

Clock:
Center
↓
Date
```

### Landscape

```text
Orientation = Landscape

Large Clock
↓
Center
↓
Date
```

### Tablet/Desktop

Clock dapat diperbesar selama:

```text
No Overflow
No Clipping
No Distortion
```

---

# 24. Flutter Architecture

Recommended structure:

```text
lib/
├── features/
│   └── clock/
│       ├── presentation/
│       │   ├── pages/
│       │   │   └── clock_page.dart
│       │   │
│       │   └── widgets/
│       │       ├── flip_clock.dart
│       │       ├── flip_digit.dart
│       │       ├── flip_card.dart
│       │       ├── flip_separator.dart
│       │       └── clock_date.dart
│       │
│       └── logic/
│           └── clock_controller.dart
│
└── core/
    └── theme/
        └── app_theme.dart
```

---

# 25. Recommended Components

### `ClockPage`

Bertanggung jawab terhadap:

* Layout.
* Orientation.
* Fullscreen.
* Responsive sizing.

### `FlipClock`

Bertanggung jawab terhadap:

* HH.
* MM.
* SS.
* Separator.

### `FlipDigit`

Bertanggung jawab terhadap:

* Current value.
* Previous value.
* Animation.

### `FlipCard`

Bertanggung jawab terhadap:

* Top half.
* Bottom half.
* Divider.
* Shadow.
* Neumorphic appearance.

### `ClockDate`

Bertanggung jawab terhadap:

* Current date.
* Date formatting.

### `ClockController`

Bertanggung jawab terhadap:

* System time.
* Timer.
* Updating clock.
* Detecting changed digits.

---

# 26. State Management

Untuk MVP, tidak diperlukan state management kompleks.

Gunakan:

```text
Timer.periodic()
```

untuk update waktu.

Gunakan:

```text
AnimationController
```

untuk flip animation.

Pastikan:

```text
Timer
```

dan:

```text
AnimationController
```

di-dispose ketika halaman dihancurkan.

Jika DashKey sudah menggunakan Riverpod, Bloc, atau state management lain, Clock dapat mengikuti architecture yang sudah digunakan.

---

# 27. Performance

Clock harus berjalan smooth.

Requirements:

* Target 60 FPS.
* Tidak rebuild seluruh halaman setiap detik jika tidak diperlukan.
* Hanya digit yang berubah yang melakukan animation.
* Timer harus dihentikan ketika page dispose.
* AnimationController harus di-dispose.
* Tidak boleh terjadi memory leak.
* Tidak boleh terjadi frame drop yang terlihat.

---

# 28. Acceptance Criteria

## AC-01 — Display

Ketika Clock Page dibuka:

```text
HH : MM : SS
```

harus menampilkan waktu sistem saat ini.

---

## AC-02 — Realtime

Clock harus update setiap satu detik.

---

## AC-03 — Flip

Ketika digit berubah, digit tersebut melakukan flip animation.

---

## AC-04 — Unchanged Digit

Digit yang tidak berubah tidak melakukan animation.

---

## AC-05 — Portrait

Portrait harus:

* Responsive.
* Centered.
* Tidak overflow.
* Tidak terpotong.

---

## AC-06 — Landscape

Landscape harus:

* Menggunakan ukuran clock lebih besar.
* Centered.
* Tidak menggunakan sidebar.
* Tidak menggunakan bottom navigation.

---

## AC-07 — Fullscreen

Clock dapat ditampilkan fullscreen tanpa system UI.

---

## AC-08 — Date

Tanggal ditampilkan di bawah clock.

---

## AC-09 — Lifecycle

Timer dan animation controller dihentikan ketika user meninggalkan Clock Page.

---

# 29. MVP Scope

### Included

* [x] Flip Clock.
* [x] HH : MM : SS.
* [x] Realtime system time.
* [x] Flip animation.
* [x] Dark Neumorphism.
* [x] Portrait.
* [x] Landscape.
* [x] Responsive sizing.
* [x] Date.
* [x] Fullscreen.

### Not Included

* [ ] Stopwatch.
* [ ] Countdown.
* [ ] Alarm.
* [ ] World Clock.
* [ ] Weather.
* [ ] Multiple Clock Theme.
* [ ] Custom Background.
* [ ] Custom Animation.
* [ ] 12-hour format.

---

# 30. Future Development

Clock dapat dikembangkan menjadi:

```text
Clock
├── Flip Clock
├── Digital Clock
├── Analog Clock
├── Countdown
├── Stopwatch
└── World Clock
```

Theme:

```text
Clock Theme
├── Dark Neumorphism
├── Light Neumorphism
├── Classic Flip
├── Minimal
└── Cyber
```

---

# 31. Final Design Principle

DashKey Clock harus terasa seperti **special mode**, bukan halaman aplikasi biasa.

Prinsip utama:

```text
Minimal
   +
Large Typography
   +
Mechanical Flip
   +
Dark Neumorphism
   +
Smooth Animation
   +
Responsive Layout
```

**Primary Focus:**

> The clock must always be the visual center of the screen.

Jangan menambahkan elemen UI yang tidak diperlukan pada Clock Mode.

Tujuan akhirnya adalah membuat DashKey memiliki **fullscreen ambient clock** yang terlihat premium, modern, dan cocok digunakan pada mobile, tablet, maupun desktop.
