/// Konfigurasi feedback tombol (PRD prdbutton.md §9–§10, §14).
///
/// Menyimpan profil feedback default per jenis interaksi:
/// Soft, Physical (default DashKey), Mechanical, Minimal, Silent.
library;

/// Jenis haptic yang didukung (PRD §5).
enum DashHaptic { none, light, medium, heavy, selection }

/// Profil feedback tombol (PRD §9).
enum FeedbackProfile {
  soft,
  physical,
  mechanical,
  minimal,
  silent;

  String get label => switch (this) {
        FeedbackProfile.soft => 'Soft',
        FeedbackProfile.physical => 'Physical',
        FeedbackProfile.mechanical => 'Mechanical',
        FeedbackProfile.minimal => 'Minimal',
        FeedbackProfile.silent => 'Silent',
      };
}

/// Pengaturan feedback yang dipakai engine (persisten di perangkat).
class FeedbackSettings {
  const FeedbackSettings({
    this.profile = FeedbackProfile.physical,
    this.hapticEnabled = true,
    this.hapticIntensity = 1.0, // 0.5..1.5
    this.animationEnabled = true,
    this.animationSpeed = 1.0, // 0.5..1.5
    this.soundEnabled = false,
    this.soundVolume = 1.0, // 0..1 (cadangan untuk custom sound)
    this.longPressClose = true,
    this.longPressMs = 600,
  });

  final FeedbackProfile profile;
  final bool hapticEnabled;
  final double hapticIntensity;
  final bool animationEnabled;
  final double animationSpeed;
  final bool soundEnabled;
  final double soundVolume;

  /// Long press pada tombol = menutup aplikasi (default global).
  final bool longPressClose;

  /// Durasi tahan sebelum close di-trigger (ms).
  final int longPressMs;

  /// Nilai default profil (PRD §9, §10).
  static const defaults = {
    FeedbackProfile.soft: FeedbackSettings(
      profile: FeedbackProfile.soft,
      hapticIntensity: 0.7,
      animationSpeed: 1.1,
    ),
    FeedbackProfile.physical: FeedbackSettings(
      profile: FeedbackProfile.physical,
      hapticIntensity: 1.0,
      animationSpeed: 1.0,
    ),
    FeedbackProfile.mechanical: FeedbackSettings(
      profile: FeedbackProfile.mechanical,
      hapticIntensity: 1.4,
      animationSpeed: 0.85,
    ),
    FeedbackProfile.minimal: FeedbackSettings(
      profile: FeedbackProfile.minimal,
      hapticIntensity: 0.6,
      animationSpeed: 1.3,
    ),
    FeedbackProfile.silent: FeedbackSettings(
      profile: FeedbackProfile.silent,
      hapticEnabled: false,
      soundEnabled: false,
      animationSpeed: 1.4,
    ),
  };

  FeedbackSettings copyWith({
    FeedbackProfile? profile,
    bool? hapticEnabled,
    double? hapticIntensity,
    bool? animationEnabled,
    double? animationSpeed,
    bool? soundEnabled,
    double? soundVolume,
    bool? longPressClose,
    int? longPressMs,
  }) =>
      FeedbackSettings(
        profile: profile ?? this.profile,
        hapticEnabled: hapticEnabled ?? this.hapticEnabled,
        hapticIntensity: hapticIntensity ?? this.hapticIntensity,
        animationEnabled: animationEnabled ?? this.animationEnabled,
        animationSpeed: animationSpeed ?? this.animationSpeed,
        soundEnabled: soundEnabled ?? this.soundEnabled,
        soundVolume: soundVolume ?? this.soundVolume,
        longPressClose: longPressClose ?? this.longPressClose,
        longPressMs: longPressMs ?? this.longPressMs,
      );

  Map<String, dynamic> toJson() => {
        'profile': profile.name,
        'hapticEnabled': hapticEnabled,
        'hapticIntensity': hapticIntensity,
        'animationEnabled': animationEnabled,
        'animationSpeed': animationSpeed,
        'soundEnabled': soundEnabled,
        'soundVolume': soundVolume,
        'longPressClose': longPressClose,
        'longPressMs': longPressMs,
      };

  factory FeedbackSettings.fromJson(Map<String, dynamic> json) =>
      FeedbackSettings(
        profile: FeedbackProfile.values.firstWhere(
          (p) => p.name == json['profile'],
          orElse: () => FeedbackProfile.physical,
        ),
        hapticEnabled: json['hapticEnabled'] as bool? ?? true,
        hapticIntensity: (json['hapticIntensity'] as num?)?.toDouble() ?? 1.0,
        animationEnabled: json['animationEnabled'] as bool? ?? true,
        animationSpeed: (json['animationSpeed'] as num?)?.toDouble() ?? 1.0,
        soundEnabled: json['soundEnabled'] as bool? ?? false,
        soundVolume: (json['soundVolume'] as num?)?.toDouble() ?? 1.0,
        longPressClose: json['longPressClose'] as bool? ?? true,
        longPressMs: json['longPressMs'] as int? ?? 600,
      );
}
