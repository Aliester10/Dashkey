/// Config Editor Controller — draft config lokal yang diedit di HP,
/// lalu dikirim ke Host via `save_config` (Fase 6, FR-7..15).
///
/// Strategi: draft disimpan sebagai JSON Map (deep copy dari config
/// ter-sync). Host memvalidasi & menyimpan; setelah sukses, config_sync
/// broadcast akan merefresh draft (jika tidak sedang dirty).
library;

import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/protocol/messages.dart';
import '../../connection/controllers/connection_controller.dart';
import '../../connection/providers/connection_providers.dart';

/// State editor config.
class ConfigEditorState {
  const ConfigEditorState({
    this.draft,
    this.dirty = false,
    this.saving = false,
    this.error,
    this.lastSuccess,
    this.lastSfx,
  });

  /// Draft JSON config; null jika belum ada editor yang dibuka.
  final Map<String, dynamic>? draft;
  final bool dirty;
  final bool saving;
  final String? error;
  final String? lastSuccess;

  /// Hasil import SFX terakhir (untuk snackbar).
  final ({bool success, String message})? lastSfx;

  ConfigEditorState copyWith({
    Map<String, dynamic>? draft,
    bool? dirty,
    bool? saving,
    String? error,
    String? lastSuccess,
    ({bool success, String message})? lastSfx,
  }) =>
      ConfigEditorState(
        draft: draft ?? this.draft,
        dirty: dirty ?? this.dirty,
        saving: saving ?? this.saving,
        error: error,
        lastSuccess: lastSuccess,
        lastSfx: lastSfx,
      );
}

final configEditorProvider =
    NotifierProvider<ConfigEditorController, ConfigEditorState>(
  ConfigEditorController.new,
);

class ConfigEditorController extends Notifier<ConfigEditorState> {
  @override
  ConfigEditorState build() {
    // Sinkronkan draft dari config yang diterima Host.
    ref.listen(connectionControllerProvider, (prev, next) {
      final config = next.config;
      if (config == null) return;
      if (state.dirty) return; // jangan timpa edit yang belum disimpan
      final draft = jsonDecode(jsonEncode(config.toJson()))
          as Map<String, dynamic>;
      state = state.copyWith(draft: draft);
    });

    // Tangani hasil save_config dari Host.
    ref.listen<AsyncValue<ProtocolMessage>>(wsMessagesProvider, (prev, next) {
      next.whenData((msg) {
        if (msg.type == 'config_saved') {
          final success = msg.payload?['success'] as bool? ?? false;
          final message = msg.payload?['message'] as String? ?? '';
          state = state.copyWith(
            saving: false,
            dirty: success ? false : state.dirty,
            error: success ? null : message,
            lastSuccess: success ? message : null,
          );
        } else if (msg.type == 'sfx_imported') {
          final success = msg.payload?['success'] as bool? ?? false;
          final message = msg.payload?['message'] as String? ?? '';
          state = state.copyWith(lastSfx: (success: success, message: message));
          if (success) {
            // Muat ulang draft dari config ter-sync (tombol SFX baru masuk).
            final config = ref.read(connectionControllerProvider).config;
            if (config != null) {
              final fresh = jsonDecode(jsonEncode(config.toJson()))
                  as Map<String, dynamic>;
              state = state.copyWith(draft: fresh, dirty: false);
            }
          }
        }
      });
    });

    return const ConfigEditorState();
  }

  /// Buka editor: inisialisasi draft dari config aktif.
  void openEditor() {
    final config = ref.read(connectionControllerProvider).config;
    if (config == null) return;
    final draft = jsonDecode(jsonEncode(config.toJson()))
        as Map<String, dynamic>;
    state = state.copyWith(draft: draft, dirty: false, error: null);
  }

  Map<String, dynamic> _draft() {
    final d = state.draft;
    if (d == null) throw StateError('editor belum dibuka');
    return d;
  }

  void _touch() => state = state.copyWith(dirty: true);

  static String _newId(String prefix) =>
      '$prefix${DateTime.now().millisecondsSinceEpoch}';

  // ---- Tombol ----

  /// Tambah tombol kosong ke page [pageId].
  String addButton(String pageId) {
    final draft = _draft();
    final buttonId = _newId('btn_');
    final button = {
      'button_id': buttonId,
      'label': 'Tombol Baru',
      'color': '#1E88E5',
      'actions': <Map<String, dynamic>>[],
    };
    (draft['buttons'] as Map<String, dynamic>)[buttonId] = button;
    final page = (draft['pages'] as Map<String, dynamic>)[pageId]
        as Map<String, dynamic>;
    (page['buttons'] as List).add(buttonId);
    _touch();
    return buttonId;
  }

  /// Perbarui satu tombol (JSON lengkap).
  void updateButton(Map<String, dynamic> button) {
    final draft = _draft();
    final buttonId = button['button_id'] as String;
    (draft['buttons'] as Map<String, dynamic>)[buttonId] = button;
    _touch();
  }

  /// Hapus tombol dari map dan semua page.
  void deleteButton(String buttonId) {
    final draft = _draft();
    (draft['buttons'] as Map<String, dynamic>).remove(buttonId);
    for (final page in (draft['pages'] as Map<String, dynamic>).values) {
      final list = (page as Map<String, dynamic>)['buttons'] as List;
      list.remove(buttonId);
    }
    _touch();
  }

  /// Ubah posisi tombol di dalam page.
  void moveButton(String pageId, int oldIndex, int newIndex) {
    final draft = _draft();
    final page = (draft['pages'] as Map<String, dynamic>)[pageId]
        as Map<String, dynamic>;
    final list = page['buttons'] as List;
    if (oldIndex < 0 || oldIndex >= list.length) return;
    if (newIndex > oldIndex) newIndex -= 1;
    final item = list.removeAt(oldIndex);
    list.insert(newIndex.clamp(0, list.length), item);
    _touch();
  }

  // ---- Page ----

  /// Tambah page baru ke profile aktif dan jadikan aktif.
  String addPage() {
    final draft = _draft();
    final pageId = _newId('page_');
    (draft['pages'] as Map<String, dynamic>)[pageId] = {
      'page_id': pageId,
      'name': 'Page Baru',
      'grid_size': {'rows': 3, 'cols': 3},
      'buttons': <String>[],
    };
    final activeProfile = draft['active_profile'] as String;
    for (final profile in draft['profiles'] as List) {
      final p = profile as Map<String, dynamic>;
      if (p['profile_id'] == activeProfile) {
        (p['pages'] as List).add(pageId);
      }
    }
    draft['active_page'] = pageId;
    _touch();
    return pageId;
  }

  /// Perbarui satu page (JSON lengkap).
  void updatePage(Map<String, dynamic> page) {
    final draft = _draft();
    (draft['pages'] as Map<String, dynamic>)[page['page_id'] as String] =
        page;
    _touch();
  }

  /// Hapus page; referensi di profile dibersihkan; active_page digeser.
  void deletePage(String pageId) {
    final draft = _draft();
    (draft['pages'] as Map<String, dynamic>).remove(pageId);
    final profiles = draft['profiles'] as List;
    String? fallback;
    for (final profile in profiles) {
      final pages = (profile as Map<String, dynamic>)['pages'] as List;
      pages.remove(pageId);
      if (pages.isNotEmpty) fallback ??= pages.first as String;
    }
    if (draft['active_page'] == pageId) {
      draft['active_page'] = fallback ?? '';
    }
    _touch();
  }

  /// Set page aktif (editor preview, tanpa kirim switch_page).
  void setActivePage(String pageId) {
    _draft()['active_page'] = pageId;
    _touch();
  }

  // ---- Profile ----

  /// Tambah profile baru dengan satu page baru.
  String addProfile() {
    final draft = _draft();
    final pageId = _newId('page_');
    final profileId = _newId('profile_');
    (draft['pages'] as Map<String, dynamic>)[pageId] = {
      'page_id': pageId,
      'name': 'Page Baru',
      'grid_size': {'rows': 3, 'cols': 3},
      'buttons': <String>[],
    };
    (draft['profiles'] as List).add({
      'profile_id': profileId,
      'name': 'Profile Baru',
      'pages': [pageId],
    });
    _touch();
    return profileId;
  }

  /// Perbarui profile (JSON lengkap).
  void updateProfile(Map<String, dynamic> profile) {
    final draft = _draft();
    final profiles = draft['profiles'] as List;
    final index = profiles.indexWhere(
      (p) => (p as Map<String, dynamic>)['profile_id'] ==
          profile['profile_id'],
    );
    if (index >= 0) profiles[index] = profile;
    _touch();
  }

  /// Hapus profile (page miliknya ikut dihapus bila tidak dipakai profile lain).
  void deleteProfile(String profileId) {
    final draft = _draft();
    final profiles = draft['profiles'] as List;
    final target = profiles
        .cast<Map<String, dynamic>>()
        .firstWhere((p) => p['profile_id'] == profileId);
    profiles.remove(target);

    // Hapus page yang tidak lagi dirujuk profile mana pun.
    final usedPages = <String>{
      for (final p in profiles.cast<Map<String, dynamic>>())
        ...(p['pages'] as List).cast<String>(),
    };
    final pages = draft['pages'] as Map<String, dynamic>;
    for (final pageId in (target['pages'] as List).cast<String>()) {
      if (!usedPages.contains(pageId)) pages.remove(pageId);
    }

    if (draft['active_profile'] == profileId && profiles.isNotEmpty) {
      draft['active_profile'] =
          (profiles.first as Map<String, dynamic>)['profile_id'];
      final firstPage =
          ((profiles.first as Map<String, dynamic>)['pages'] as List).first;
      draft['active_page'] = firstPage;
    }
    _touch();
  }

  // ---- Simpan ----

  /// Kirim seluruh draft config ke Host (`save_config`).
  void save() {
    final draft = state.draft;
    if (draft == null || state.saving) return;
    final ws = ref.read(wsConnectionProvider);
    ws.send(ProtocolMessage(type: 'save_config', payload: {'config': draft}));
    state = state.copyWith(saving: true, error: null, lastSuccess: null);
  }

  /// Batalkan edit: buang draft, muat ulang dari config ter-sync.
  void discard() {
    final config = ref.read(connectionControllerProvider).config;
    if (config == null) {
      state = state.copyWith(dirty: false);
      return;
    }
    final draft =
        jsonDecode(jsonEncode(config.toJson())) as Map<String, dynamic>;
    state = state.copyWith(draft: draft, dirty: false, error: null);
  }

  /// Kirim permintaan import SFX ke Host (URL/iframe myinstants).
  void importSfx(String input) {
    final ws = ref.read(wsConnectionProvider);
    ws.send(Outbound.importSfx(input));
  }
}
