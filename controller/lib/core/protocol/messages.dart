/// Model pesan protokol DashKey (mirror dari `host/src/protocol.rs`).
///
/// Format umum sesuai PRD section 8:
/// ```json
/// { "type": "string", "payload": { ... } }
/// ```
library;

import 'dart:convert';

/// Pesan generik dari/ke Host.
class ProtocolMessage {
  const ProtocolMessage({required this.type, this.payload});

  final String type;
  final Map<String, dynamic>? payload;

  factory ProtocolMessage.fromJson(Map<String, dynamic> json) =>
      ProtocolMessage(
        type: json['type'] as String? ?? '',
        payload: json['payload'] as Map<String, dynamic>?,
      );

  String encode() => jsonEncode({'type': type, 'payload': payload});

  @override
  String toString() => encode();
}

/// Tipe pesan keluar (Controller → Host).
class Outbound {
  Outbound._();

  static ProtocolMessage echo(String text) => ProtocolMessage(
        type: 'echo',
        payload: {'text': text},
      );

  static ProtocolMessage ping() =>
      const ProtocolMessage(type: 'ping', payload: null);

  static ProtocolMessage pairRequest({
    required String pairToken,
    required String deviceName,
  }) =>
      ProtocolMessage(
        type: 'pair_request',
        payload: {'pair_token': pairToken, 'device_name': deviceName},
      );

  static ProtocolMessage auth({
    required String deviceId,
    required String authToken,
  }) =>
      ProtocolMessage(
        type: 'auth',
        payload: {'device_id': deviceId, 'auth_token': authToken},
      );

  static ProtocolMessage buttonPress({
    required String buttonId,
    required String pageId,
  }) =>
      ProtocolMessage(
        type: 'button_press',
        payload: {'button_id': buttonId, 'page_id': pageId},
      );

  static ProtocolMessage switchPage(String pageId) => ProtocolMessage(
        type: 'switch_page',
        payload: {'page_id': pageId},
      );

  static ProtocolMessage switchProfile(String profileId) => ProtocolMessage(
        type: 'switch_profile',
        payload: {'profile_id': profileId},
      );

  /// Import SFX dari myinstants.com (URL atau kode embed iframe).
  static ProtocolMessage importSfx(String url) => ProtocolMessage(
        type: 'import_sfx',
        payload: {'url': url},
      );

  // ── PRD2 Trackpad: pesan mouse (fast path di Host) ──

  /// Gerak kursor relatif (delta piksel).
  static ProtocolMessage mouseMove({required int dx, required int dy}) =>
      ProtocolMessage(type: 'mouse_move', payload: {'dx': dx, 'dy': dy});

  /// Klik tombol mouse ("left" | "right" | "middle").
  static ProtocolMessage mouseClick(String button) => ProtocolMessage(
        type: 'mouse_click',
        payload: {'button': button},
      );

  /// Scroll vertikal (dy > 0 = bawah).
  static ProtocolMessage mouseScroll(int dy) =>
      ProtocolMessage(type: 'mouse_scroll', payload: {'dy': dy});

  /// Tekan tombol (drag: down + move + up).
  static ProtocolMessage mouseDown(String button) =>
      ProtocolMessage(type: 'mouse_down', payload: {'button': button});

  /// Lepas tombol.
  static ProtocolMessage mouseUp(String button) =>
      ProtocolMessage(type: 'mouse_up', payload: {'button': button});
}

/// Tipe pesan masuk (Host → Controller).
class Inbound {
  static const echoReply = 'echo_reply';
  static const pong = 'pong';
  static const error = 'error';
  static const pairSuccess = 'pair_success';
  static const pairError = 'pair_error';
  static const authSuccess = 'auth_success';
  static const authError = 'auth_error';
  static const statusUpdate = 'status_update';
  static const configSync = 'config_sync';
  static const actionResult = 'action_result';
  static const configSaved = 'config_saved';
  static const sfxImported = 'sfx_imported';
}
