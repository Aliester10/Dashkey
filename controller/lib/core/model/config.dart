/// Model config DashKey (mirror `host/src/config/store.rs`, PRD section 9).
///
/// Config diterima dari Host lewat `config_sync`.
library;

/// Struktur Profile (PRD 9.1).
class ProfileDef {
  const ProfileDef({
    required this.profileId,
    required this.name,
    required this.pages,
  });

  final String profileId;
  final String name;
  final List<String> pages;

  factory ProfileDef.fromJson(Map<String, dynamic> json) => ProfileDef(
        profileId: json['profile_id'] as String? ?? '',
        name: json['name'] as String? ?? '',
        pages: (json['pages'] as List?)?.cast<String>() ?? [],
      );

  Map<String, dynamic> toJson() => {
        'profile_id': profileId,
        'name': name,
        'pages': pages,
      };
}

/// Struktur GridSize (PRD 9.2).
class GridSizeDef {
  const GridSizeDef({required this.rows, required this.cols});

  final int rows;
  final int cols;

  factory GridSizeDef.fromJson(Map<String, dynamic> json) => GridSizeDef(
        rows: json['rows'] as int? ?? 4,
        cols: json['cols'] as int? ?? 4,
      );

  Map<String, dynamic> toJson() => {'rows': rows, 'cols': cols};
}

/// Struktur Page (PRD 9.2).
class PageDef {
  const PageDef({
    required this.pageId,
    required this.name,
    required this.gridSize,
    required this.buttons,
    this.pageType = 'buttons',
  });

  final String pageId;
  final String name;
  final GridSizeDef gridSize;
  final List<String> buttons;

  /// PRD2: "buttons" (grid) atau "trackpad".
  final String pageType;

  bool get isTrackpad => pageType == 'trackpad';

  factory PageDef.fromJson(Map<String, dynamic> json) => PageDef(
        pageId: json['page_id'] as String? ?? '',
        name: json['name'] as String? ?? '',
        gridSize: GridSizeDef.fromJson(
          json['grid_size'] as Map<String, dynamic>? ?? {},
        ),
        buttons: (json['buttons'] as List?)?.cast<String>() ?? [],
        pageType: json['page_type'] as String? ?? 'buttons',
      );

  Map<String, dynamic> toJson() => {
        'page_id': pageId,
        'name': name,
        'grid_size': gridSize.toJson(),
        'buttons': buttons,
        'page_type': pageType,
      };
}

/// Satu aksi pada tombol (PRD 9.3, FR-14).
class ActionDef {
  const ActionDef({
    required this.actionType,
    this.target,
    this.keys,
    this.command,
    this.control,
  });

  final String actionType;
  final String? target;
  final List<String>? keys;
  final String? command;
  final String? control;

  factory ActionDef.fromJson(Map<String, dynamic> json) => ActionDef(
        actionType: json['action_type'] as String? ?? '',
        target: json['target'] as String?,
        keys: (json['keys'] as List?)?.cast<String>(),
        command: json['command'] as String?,
        control: json['control'] as String?,
      );

  Map<String, dynamic> toJson() => {
        'action_type': actionType,
        if (target != null) 'target': target,
        if (keys != null) 'keys': keys,
        if (command != null) 'command': command,
        if (control != null) 'control': control,
      };

  /// Label pendek untuk ditampilkan di editor.
  String get summary {
    final value = target ?? command ?? control ?? (keys?.join('+')) ?? '';
    return value.isEmpty ? actionType : '$actionType: $value';
  }
}

/// Struktur Button (PRD 9.3).
class ButtonDef {
  const ButtonDef({
    required this.buttonId,
    required this.label,
    required this.color,
    this.icon,
    this.actions = const [],
    this.iconData,
  });

  final String buttonId;
  final String label;
  final String color;
  final String? icon;
  final List<ActionDef> actions;

  /// Gambar ikon (base64) di-embed oleh Host saat config_sync — agar HP
  /// menampilkan ikon identik dengan GUI desktop (icon gambar lokal PC).
  final String? iconData;

  factory ButtonDef.fromJson(Map<String, dynamic> json) => ButtonDef(
        buttonId: json['button_id'] as String? ?? '',
        label: json['label'] as String? ?? '',
        color: json['color'] as String? ?? '#1E88E5',
        icon: json['icon'] as String?,
        actions: (json['actions'] as List?)
                ?.map((a) => ActionDef.fromJson(a as Map<String, dynamic>))
                .toList() ??
            [],
        iconData: json['icon_data'] as String?,
      );

  Map<String, dynamic> toJson() => {
        'button_id': buttonId,
        'label': label,
        'color': color,
        if (icon != null) 'icon': icon,
        if (iconData != null) 'icon_data': iconData,
        'actions': actions.map((a) => a.toJson()).toList(),
      };

  /// Parse warna hex (#RRGGBB) → Color; fallback biru.
  static int parseColor(String hex) {
    final cleaned = hex.replaceFirst('#', '');
    final value = int.tryParse(cleaned, radix: 16);
    return (value ?? 0x1E88E5) | 0xFF000000;
  }
}

/// Seluruh config yang di-sync dari Host.
class ConfigData {
  const ConfigData({
    required this.profiles,
    required this.pages,
    required this.buttons,
    required this.activeProfile,
    required this.activePage,
    this.obsJson,
  });

  final List<ProfileDef> profiles;
  final Map<String, PageDef> pages;
  final Map<String, ButtonDef> buttons;
  final String activeProfile;
  final String activePage;

  /// Raw JSON bagian `obs` (disimpan agar tidak hilang saat save ulang).
  final Map<String, dynamic>? obsJson;

  factory ConfigData.fromJson(Map<String, dynamic> json) {
    final profiles = (json['profiles'] as List?)
            ?.map((p) => ProfileDef.fromJson(p as Map<String, dynamic>))
            .toList() ??
        [];
    final pages = (json['pages'] as Map<String, dynamic>?)?.map(
          (k, v) => MapEntry(k, PageDef.fromJson(v as Map<String, dynamic>)),
        ) ??
        {};
    final buttons = (json['buttons'] as Map<String, dynamic>?)?.map(
          (k, v) => MapEntry(k, ButtonDef.fromJson(v as Map<String, dynamic>)),
        ) ??
        {};
    return ConfigData(
      profiles: profiles,
      pages: pages,
      buttons: buttons,
      activeProfile: json['active_profile'] as String? ?? '',
      activePage: json['active_page'] as String? ?? '',
      obsJson: json['obs'] as Map<String, dynamic>?,
    );
  }

  Map<String, dynamic> toJson() => {
        'profiles': profiles.map((p) => p.toJson()).toList(),
        'pages': pages.map((k, v) => MapEntry(k, v.toJson())),
        'buttons': buttons.map((k, v) => MapEntry(k, v.toJson())),
        'active_profile': activeProfile,
        'active_page': activePage,
        if (obsJson != null) 'obs': obsJson,
      };

  /// Page aktif; null jika config kosong.
  PageDef? get currentPage => pages[activePage];

  /// Tombol-tombol milik page aktif, berurutan.
  List<ButtonDef> get currentButtons {
    final page = currentPage;
    if (page == null) return [];
    return page.buttons
        .map((id) => buttons[id])
        .whereType<ButtonDef>()
        .toList();
  }
}
