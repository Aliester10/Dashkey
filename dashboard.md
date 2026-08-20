Tentu. Untuk **DashKey Host**, saya sarankan kita jadikan desain dark mode yang tadi sebagai **design system resmi**, supaya nanti ketika kamu membuat halaman Buttons, Profiles, Devices, Pairing, Settings, dll., semuanya konsisten.

Di bawah ini saya buat spesifikasi yang cukup detail untuk langsung dijadikan acuan implementasi di Rust GUI.

# DashKey Host — Dark Neumorphic Design Specification

## 1. Design Direction

### Nama gaya

**Dark Neumorphic Command Center**

### Karakter visual

Desain harus terasa:

* Futuristik
* Premium
* Minimalis
* Soft
* Modern
* Desktop-oriented
* Tidak terlalu “gaming”
* Tidak terlalu colorful
* Memiliki sedikit efek glowing
* Menggunakan **dark neumorphism**, bukan glassmorphism murni
* Fokus pada readability dan usability

Konsep utamanya:

> **Dark surface + soft elevation + subtle glow + purple identity**

DashKey harus terasa seperti aplikasi profesional untuk mengontrol perangkat, bukan dashboard website biasa.

---

# 2. Overall Layout

Ukuran referensi desain:

**1280 × 1168 px**

Namun layout harus **responsive** sehingga tetap nyaman pada:

* 1280 × 720
* 1366 × 768
* 1440 × 900
* 1920 × 1080

### Struktur utama

```text
┌─────────────────────────────────────────────────────────────┐
│                       Window Header                         │
├─────────────────────────────────────────────────────────────┤
│ Brand                         Navigation          Status    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                    Welcome / Hero Card                      │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Device      Profile       Page        Button                │
│   Card        Card         Card         Card                 │
│                                                             │
├──────────────────────────────┬──────────────────────────────┤
│                              │                              │
│         Quick Start          │       Recent Activity        │
│                              │                              │
├──────────────────────────────┴──────────────────────────────┤
│                    Host Status Bar                          │
├─────────────────────────────────────────────────────────────┤
│                    Bottom Information                       │
└─────────────────────────────────────────────────────────────┘
```

---

# 3. Application Background

Background merupakan fondasi seluruh neumorphic design.

### Main background

```text
#11151F
```

Alternatif gradient sangat subtle:

```text
Top:    #141925
Bottom: #0F131C
```

Jangan menggunakan hitam murni `#000000`.

Alasannya karena neumorphism membutuhkan perbedaan luminance antara background dan surface.

### Background character

Background harus:

* sangat gelap
* sedikit kebiruan
* tidak flat black
* tidak menggunakan texture
* tidak menggunakan gradient yang terlalu terlihat

---

# 4. Surface / Card

Semua card menggunakan dark elevated surface.

### Card background

```text
#1A202C
```

atau:

```text
#1B2130
```

### Border

```text
rgba(255,255,255,0.06)
```

Border jangan terlalu terang.

---

# 5. Neumorphism

Ini merupakan bagian paling penting.

Jangan membuat shadow seperti website biasa.

Gunakan **dua arah shadow**:

### Dark shadow

```text
rgba(0,0,0,0.45)
```

Contoh:

```text
offset X: 0
offset Y: 8
blur: 20
spread: -4
```

### Soft highlight

```text
rgba(255,255,255,0.035)
```

Contoh:

```text
offset X: -2
offset Y: -2
blur: 8
```

Tujuannya menghasilkan:

```text
        soft highlight
             ↓
      ┌──────────────┐
     /                \
    │      CARD        │
     \                /
      └──────────────┘
             ↓
        dark shadow
```

Bukan:

```text
████████████
hard shadow
```

---

# 6. Border Radius

Gunakan radius yang konsisten.

### Main cards

```text
16px
```

### Hero card

```text
16px
```

### Statistic cards

```text
16px
```

### Buttons

```text
12px
```

### Icon container

```text
12px
```

### Status pill

```text
999px
```

### Window

```text
18px
```

Hindari terlalu banyak radius `24–32px` karena akan membuat aplikasi terlihat seperti mobile UI.

---

# 7. Typography

Font harus modern dan mudah dibaca.

Rekomendasi:

### Primary

**Inter**

Fallback:

```text
Inter, SF Pro Display, Segoe UI, sans-serif
```

Jika environment Rust GUI tidak menyediakan Inter:

```text
Segoe UI
```

atau:

```text
Noto Sans
```

---

## Typography scale

### Application title

```text
20px
weight: 600
```

Contoh:

**DashKey**

---

### Page title

```text
28px
weight: 600
```

Contoh:

**Selamat datang di DashKey**

---

### Section title

```text
18px
weight: 500
```

Contoh:

**Quick Start**

---

### Card label

```text
12px
weight: 500
uppercase
letter spacing: 0.3px
```

Contoh:

```text
DEVICE ONLINE
```

---

### Statistic number

```text
32px
weight: 500–600
```

Contoh:

```text
1
```

---

### Body

```text
14px
weight: 400
```

---

### Secondary text

```text
12px
weight: 400
```

Color:

```text
#7F8798
```

---

# 8. Primary Color

DashKey identity menggunakan **electric purple**.

Primary:

```text
#8B5CF6
```

Bright:

```text
#A78BFA
```

Deep:

```text
#6D4AFF
```

Glow:

```text
rgba(139,92,246,0.35)
```

---

# 9. Accent Colors

Jangan semua elemen menggunakan purple.

Gunakan accent berdasarkan konteks.

### Device

```text
#34D399
```

Green.

### Profile

```text
#3B82F6
```

Blue.

### Page

```text
#8B5CF6
```

Purple.

### Button

```text
#F59E0B
```

Orange.

Ini membuat dashboard mudah dipindai secara visual.

---

# 10. Top Window Header

Bagian paling atas:

```text
DashKey Host
```

posisi:

**center**

Window controls:

```text
—   □   ×
```

di kanan.

### Window controls

Ukuran sekitar:

```text
32 × 32 px
```

Normal:

```text
background: transparent
```

Hover:

```text
rgba(255,255,255,0.06)
```

Close hover:

```text
rgba(239,68,68,0.15)
```

---

# 11. Application Header

Header terdiri dari:

```text
DashKey    ● 1 device
```

di kiri.

Kemudian navigation.

Di kanan:

```text
● 192.168.1.33:48484  •  Nuel
```

### Brand icon

Ukuran:

```text
36 × 36
```

Background:

```text
#6D4AFF
```

Icon:

```text
white
```

Border radius:

```text
10–12px
```

Tambahkan purple glow:

```text
0 0 15px rgba(139,92,246,.25)
```

---

# 12. Navigation

Navigation:

```text
Dashboard
Buttons
Profiles
Pairing
Devices
Integrations
Activity
Settings
```

### Icon

Ukuran:

```text
16–18px
```

### Text

```text
14px
```

### Inactive

```text
#8C94A6
```

### Active

Text:

```text
#A78BFA
```

Background:

```text
rgba(139,92,246,0.15)
```

Glow:

```text
0 4px 18px rgba(139,92,246,0.15)
```

### Active navigation

Contoh:

```text
┌────────────────────┐
│  ◉  Dashboard      │
└────────────────────┘
```

Radius:

```text
10px
```

---

# 13. Welcome Hero Card

Hero card merupakan elemen visual utama.

### Height

Sekitar:

```text
160–180px
```

### Content

Kiri:

```text
[ ⚡ ]

Selamat datang di DashKey

Command center untuk mengelola tombol,
device, dan integrasi PC.
```

Kanan:

Ilustrasi device / Stream Deck.

---

## Hero icon

Ukuran:

```text
64 × 64
```

Background:

```text
#5B45C7
```

Gradient:

```text
#7C5CFF → #5B45C7
```

Glow:

```text
0 0 25px rgba(124,92,255,.35)
```

---

# 14. Hero Device Illustration

Perangkat DashKey di kanan harus terlihat:

* dark
* sedikit 3D
* rounded
* purple underside glow
* tidak terlalu besar
* sedikit rotated / perspective

Warna:

```text
#171C28
```

Button:

```text
#252C3A
```

Edge:

```text
#303849
```

Purple glow:

```text
rgba(139,92,246,.8)
```

Jangan membuat ilustrasi terlalu terang.

---

# 15. Statistic Cards

Ini bagian yang sebelumnya kita utak-atik.

Menurut saya **versi kedua yang kamu pilih adalah baseline terbaik**.

Struktur:

```text
┌───────────────────────────────┐
│                               │
│       [ ICON ]                │
│                               │
│  DEVICE ONLINE                │
│                               │
│  1                  Devices   │
│                               │
└───────────────────────────────┘
```

Empat card:

```text
Device Online
Profile
Page
Button
```

---

## Card dimensions

Desktop:

```text
width: ~280px
height: ~220px
```

Dengan gap:

```text
16–20px
```

Semua card memiliki tinggi yang sama.

---

# 16. Statistic Icon

Icon container:

```text
64 × 64
```

atau sekitar:

```text
56 × 56
```

Circle / rounded container.

### Device

```text
background: rgba(16,185,129,.12)
border: rgba(52,211,153,.18)
```

Icon:

```text
#34D399
```

Glow:

```text
0 0 20px rgba(52,211,153,.20)
```

---

### Profile

```text
background: rgba(59,130,246,.12)
```

Icon:

```text
#60A5FA
```

---

### Page

```text
background: rgba(139,92,246,.12)
```

Icon:

```text
#A78BFA
```

---

### Button

```text
background: rgba(245,158,11,.12)
```

Icon:

```text
#F59E0B
```

---

# 17. Statistic Number

Angka merupakan informasi paling penting.

Misalnya:

```text
1
```

Gunakan:

```text
32px
weight: 500
color: #F1F5F9
```

Jangan menggunakan font terlalu bold.

---

# 18. Statistic Card Footer

Bagian kanan bawah:

```text
▣ Devices
```

atau:

```text
♙ Profiles
```

Text:

```text
#8A93A6
```

Font:

```text
12px
```

Icon:

```text
14px
```

Ini berfungsi sebagai contextual hint, bukan informasi utama.

---

# 19. Quick Start

Quick Start berada di kiri.

Card:

```text
Quick Start

Mulai dari pairing HP, lalu tambahkan
aplikasi sebagai shortcut.
```

Kemudian tiga action.

### Action 1

```text
🔗  Pair device baru                       >
```

### Action 2

```text
⊞   Kelola tombol                          >
```

### Action 3

```text
🔊  Integrasi OBS & soundboard             >
```

---

# 20. Quick Start Item

Ukuran:

```text
height: 62px
```

Radius:

```text
12px
```

Background:

```text
#1D2432
```

Border:

```text
rgba(255,255,255,.04)
```

Shadow:

```text
0 4px 12px rgba(0,0,0,.25)
```

---

## Hover

Ketika mouse masuk:

```text
background:
#222A3A
```

Border:

```text
rgba(139,92,246,.25)
```

Icon glow meningkat.

Arrow:

```text
#A78BFA
```

Transform:

```text
translateX(2px)
```

Transition:

```text
150–200ms
```

---

# 21. Recent Activity

Card sebelah kanan.

Header:

```text
Recent Activity
```

Activity list.

Contoh:

```text
● QR pairing baru dibuat                  Just now

● Akses device-xxxx dicabut              1m ago

● Akses device-xxxx dicabut              2m ago
```

---

# 22. Activity Indicator

Gunakan dot kecil.

```text
8 × 8px
```

Purple:

```text
#8B5CF6
```

Glow:

```text
0 0 8px rgba(139,92,246,.5)
```

---

# 23. Activity Divider

Gunakan garis yang sangat subtle:

```text
rgba(255,255,255,.05)
```

Jangan menggunakan:

```text
#444
```

karena terlalu keras.

---

# 24. Bottom Host Status

Bagian bawah:

```text
✓ Host berjalan normal
192.168.1.33:48484
•
uptime 2m 52s

                           Broadcast config
```

Card memanjang full width.

---

## Status icon

Green:

```text
#34D399
```

Container:

```text
40 × 40px
```

Glow ringan.

---

# 25. Broadcast Config Button

Button:

```text
Broadcast config
```

Background:

```text
rgba(139,92,246,.08)
```

Border:

```text
rgba(139,92,246,.35)
```

Text:

```text
#A78BFA
```

Radius:

```text
999px
```

Hover:

```text
rgba(139,92,246,.16)
```

---

# 26. Footer

Footer sangat minimal.

Kiri:

```text
QR pairing baru dibuat (berlaku 2 menit)
```

Kanan:

```text
uptime 2m 52s • 4 page • 19 tombol
```

Color:

```text
#6F788A
```

Font:

```text
11–12px
```

---

# 27. Spacing System

Gunakan spacing konsisten berdasarkan kelipatan 4.

```text
4px
8px
12px
16px
20px
24px
32px
40px
48px
```

Rekomendasi:

### Page padding

```text
24px
```

### Section gap

```text
20px
```

### Card internal padding

```text
24px
```

### Card gap

```text
16px
```

### Navigation gap

```text
8px
```

---

# 28. Shadow System

Buat beberapa level.

### Elevation 1

Untuk button kecil:

```text
0 2px 6px rgba(0,0,0,.25)
```

### Elevation 2

Untuk card:

```text
0 8px 20px rgba(0,0,0,.30)
```

### Elevation 3

Untuk floating component:

```text
0 12px 32px rgba(0,0,0,.40)
```

### Glow

Purple:

```text
0 0 20px rgba(139,92,246,.20)
```

Green:

```text
0 0 20px rgba(52,211,153,.18)
```

Blue:

```text
0 0 20px rgba(59,130,246,.18)
```

Orange:

```text
0 0 20px rgba(245,158,11,.18)
```

---

# 29. Interaction States

Setiap interactive component minimal mempunyai:

```text
Default
Hover
Pressed
Focus
Disabled
```

### Hover

Jangan mengubah warna secara drastis.

Contoh:

```text
Default:
#1A202C

Hover:
#202737
```

### Pressed

Berikan efek **inset shadow**.

```text
inset 0 2px 6px rgba(0,0,0,.35)
```

Ini sangat cocok dengan neumorphism.

---

# 30. Button Style

Primary button:

```text
background: #7655E8
color: #FFFFFF
```

Shadow:

```text
0 4px 16px rgba(118,85,232,.25)
```

Hover:

```text
#8568F5
```

Pressed:

```text
inset 0 2px 6px rgba(0,0,0,.3)
```

---

# 31. Input Fields

Untuk halaman lain seperti:

* Create Button
* Edit Profile
* Pairing
* Settings

gunakan:

```text
height: 42–44px
radius: 10px
background: #151B26
border: rgba(255,255,255,.07)
```

Text:

```text
#E5E7EB
```

Placeholder:

```text
#687286
```

Focus:

```text
border: rgba(139,92,246,.6)
box-shadow:
0 0 0 3px rgba(139,92,246,.10)
```

---

# 32. Modal / Dialog

Modal harus mengikuti neumorphic style.

Background:

```text
#1A202C
```

Overlay:

```text
rgba(5,8,15,.72)
```

Radius:

```text
18px
```

Shadow:

```text
0 20px 60px rgba(0,0,0,.55)
```

Header:

```text
18px / 600
```

Body:

```text
14px
```

---

# 33. Status Colors

Gunakan semantic color.

| Status  | Color     |
| ------- | --------- |
| Success | `#34D399` |
| Info    | `#60A5FA` |
| Primary | `#8B5CF6` |
| Warning | `#F59E0B` |
| Error   | `#F87171` |
| Neutral | `#94A3B8` |

Jangan menggunakan warna status terlalu terang memenuhi seluruh card.

Gunakan hanya untuk:

* icon
* indicator
* border kecil
* badge
* text penting

---

# 34. Iconography

Gunakan satu icon library saja.

Rekomendasi:

**Lucide Icons**

Style:

```text
stroke-based
1.5–2px
```

Jangan mencampur:

* Font Awesome
* Material Icons
* Lucide
* custom SVG

dalam satu halaman.

---

# 35. Animation

Animation harus **sangat halus**.

### Standard transition

```text
150ms ease-out
```

### Card hover

```text
200ms ease
```

### Modal

```text
200–250ms
```

### Page transition

```text
150–200ms
```

Hindari:

* bounce
* elastic
* animation terlalu lama
* card bergerak terlalu jauh

DashKey adalah **control application**, bukan game UI.

---

# 36. Neumorphism Rules

Ini penting supaya implementasi tidak berubah menjadi desain lain.

### DO

* Dark surface
* Soft shadows
* Subtle highlight
* Rounded cards
* Low contrast borders
* Small colored glow
* Consistent spacing
* Minimal animation

### DON'T

Jangan menggunakan:

```text
❌ excessive glass blur
❌ excessive transparency
❌ huge gradients
❌ hard shadows
❌ bright neon everywhere
❌ giant text
❌ excessive rounded corners
❌ colorful backgrounds
❌ excessive borders
```

---

# 37. Color Token

Kalau kamu implementasikan sebagai design system, saya sarankan membuat token seperti ini:

```text
Background
    bg-primary       #11151F
    bg-secondary     #151B26
    bg-surface       #1A202C
    bg-surface-hover #202737

Text
    text-primary     #F1F5F9
    text-secondary   #A0A8B8
    text-muted       #6F788A

Border
    border-subtle    rgba(255,255,255,0.06)
    border-active    rgba(139,92,246,0.40)

Brand
    primary          #8B5CF6
    primary-light    #A78BFA
    primary-dark     #6D4AFF

Semantic
    success          #34D399
    info             #60A5FA
    warning          #F59E0B
    danger           #F87171
```

---

# 38. Responsive Behavior

Pada layar lebar:

```text
4 statistic cards
```

Pada layar sedang:

```text
2 × 2 statistic cards
```

Pada layar kecil:

```text
1 × 4
```

Quick Start + Activity:

```text
Desktop:
2 columns

Tablet:
2 columns

Small:
1 column
```

---

# 39. Recommended Desktop Minimum

Karena ini aplikasi desktop Rust, saya sarankan:

```text
Minimum width: 1100px
Minimum height: 650px
```

Ideal:

```text
1366 × 768
```

Jangan memaksa layout desktop penuh pada ukuran 800px karena navigation akan menjadi terlalu padat.

---

# 40. Component Hierarchy

Struktur component-nya bisa dibuat seperti:

```text
DashKeyHost
│
├── WindowHeader
│
├── AppHeader
│   ├── Brand
│   ├── DeviceStatus
│   ├── Navigation
│   └── NetworkStatus
│
├── Dashboard
│   ├── WelcomeCard
│   │   ├── WelcomeIcon
│   │   ├── WelcomeText
│   │   └── DeviceIllustration
│   │
│   ├── StatsGrid
│   │   ├── DeviceOnlineCard
│   │   ├── ProfileCard
│   │   ├── PageCard
│   │   └── ButtonCard
│   │
│   ├── DashboardGrid
│   │   ├── QuickStartCard
│   │   │   ├── QuickStartItem
│   │   │   ├── QuickStartItem
│   │   │   └── QuickStartItem
│   │   │
│   │   └── RecentActivityCard
│   │       └── ActivityList
│   │
│   └── HostStatus
│
└── Footer
```

---

# 41. Halaman-Halaman Selanjutnya

Design dashboard ini sebaiknya menjadi **master visual language** untuk semua halaman DashKey.

### Buttons

Gunakan:

```text
Page Header
+ Create Button
Search
Filter
Button Grid/Table
```

### Profiles

```text
Profile Header
Profile Cards
Active Profile
Actions
```

### Pairing

Gunakan:

```text
QR Code
Pairing Code
Connected Devices
Security Status
```

### Devices

Gunakan:

```text
Device Cards
Online/Offline
Last Seen
IP Address
Actions
```

### Integrations

Gunakan:

```text
OBS
Soundboard
Media
System Controls
```

### Activity

Gunakan:

```text
Timeline
Filter
Event Type
Timestamp
Device
```

### Settings

Gunakan:

```text
General
Appearance
Network
Security
Integrations
About
```

Semua tetap memakai **surface, shadow, typography, spacing, radius, dan color token yang sama**.

---

# 42. Filosofi Utama DashKey

Kalau nanti kamu meminta AI lain atau developer lain untuk membuat halaman DashKey, saya sarankan selalu berikan prinsip ini:

> **DashKey is a professional desktop command center, not a generic admin dashboard. The UI should use dark neumorphism with subtle purple glow, soft elevation, low-contrast surfaces, restrained accent colors, generous spacing, and minimal animations. Every component should feel tactile and physical, as if the user is interacting with a premium hardware control panel.**

Dan yang paling penting:

**Jangan mengejar efek sebanyak mungkin.**

Versi yang tadi kamu pilih justru bagus karena efek neumorphism-nya **cukup terasa tetapi tidak berlebihan**. Untuk DashKey, saya akan mempertahankan prinsip:

**80% usability + 15% neumorphism + 5% futuristic glow.**

Itu menurut saya akan membuat aplikasi Rust kamu terlihat jauh lebih seperti **produk desktop profesional** daripada sekadar GUI project.
