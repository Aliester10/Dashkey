/// Connection Controller — orchestrator koneksi & sesi (riverpod 3).
///
/// Alur:
/// 1. `tryAutoConnect()` — baca kredensial tersimpan → connect → auth (FR-6).
/// 2. `pairFromQr()` — scan QR → connect → pair_request → simpan kredensial → auth.
/// 3. Pesan masuk (auth_success, config_sync, action_result, dsb) mengubah state.
library;

import 'dart:async';
import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/model/config.dart';
import '../../../core/network/ws_client.dart';
import '../../../core/protocol/messages.dart';
import '../../../core/storage/credential_store.dart';
import '../providers/connection_providers.dart';

/// Fase sesi koneksi.
enum ConnectionPhase {
  /// Belum ada kredensial; perlu pairing (scan QR).
  needsPairing,
  /// Koneksi ke Host belum terjalin.
  disconnected,
  connecting,
  /// Terhubung, menunggu autentikasi.
  authenticating,
  /// Terhubung & terautentikasi.
  authenticated,
  /// Gagal (pairing/auth/network).
  error,
}

/// State sesi koneksi.
class ConnectionState {
  const ConnectionState({
    this.phase = ConnectionPhase.disconnected,
    this.host = '',
    this.port = 48484,
    this.config,
    this.lastError,
    this.lastActionResult,
    this.buttonStates = const {},
  });

  final ConnectionPhase phase;
  final String host;
  final int port;
  final ConfigData? config;
  final String? lastError;

  /// Feedback hasil eksekusi aksi terakhir (button_id → pesan).
  final ({String buttonId, bool success, String? message})? lastActionResult;

  /// Status dinamis tombol dari Host (FR-15): button_id → (state, warna).
  final Map<String, ({String state, String? colorOverride})> buttonStates;

  ConnectionState copyWith({
    ConnectionPhase? phase,
    String? host,
    int? port,
    ConfigData? config,
    String? lastError,
    ({String buttonId, bool success, String? message})? lastActionResult,
    Map<String, ({String state, String? colorOverride})>? buttonStates,
  }) =>
      ConnectionState(
        phase: phase ?? this.phase,
        host: host ?? this.host,
        port: port ?? this.port,
        config: config ?? this.config,
        lastError: lastError,
        lastActionResult: lastActionResult ?? this.lastActionResult,
        buttonStates: buttonStates ?? this.buttonStates,
      );

  ConnectionState clearError() => copyWith(lastError: null);
}

final credentialStoreProvider = Provider<CredentialStore>((ref) {
  return CredentialStore();
});

final connectionControllerProvider =
    NotifierProvider<ConnectionController, ConnectionState>(
  ConnectionController.new,
);

class ConnectionController extends Notifier<ConnectionState> {
  WsConnection get _ws => ref.read(wsConnectionProvider);

  /// Kredensial sesi berjalan (dipakai untuk re-auth setelah reconnect).
  SavedCredentials? _sessionCreds;

  @override
  ConnectionState build() {
    // Dengarkan pesan masuk dari Host.
    ref.listen<AsyncValue<ProtocolMessage>>(wsMessagesProvider, (prev, next) {
      next.whenData(_onMessage);
    });
    // Reaksi terhadap perubahan status koneksi (auto-reconnect → re-auth).
    ref.listen<AsyncValue<WsStatus>>(wsStatusProvider, (prev, next) {
      final status = next.value;
      if (status == null) return;
      if (status == WsStatus.connected && _sessionCreds != null) {
        // Koneksi pulih (reconnect) → kirim auth ulang.
        final creds = _sessionCreds!;
        _ws.send(Outbound.auth(
          deviceId: creds.deviceId,
          authToken: creds.authToken,
        ));
        state = state.copyWith(phase: ConnectionPhase.authenticating);
      } else if (status == WsStatus.disconnected &&
          state.phase == ConnectionPhase.authenticated) {
        // Putus mendadak; WsConnection sedang mencoba reconnect.
        state = state.copyWith(phase: ConnectionPhase.connecting);
      }
    });
    // Auto-connect saat controller dibuat (bila ada kredensial).
    Future.microtask(tryAutoConnect);
    return const ConnectionState();
  }

  /// Auto-reconnect memakai kredensial tersimpan (PRD FR-6).
  Future<void> tryAutoConnect() async {
    final creds = await ref.read(credentialStoreProvider).read();
    if (creds == null) {
      state = state.copyWith(phase: ConnectionPhase.needsPairing);
      return;
    }
    await connectAndAuth(
      host: creds.host,
      port: creds.port,
      deviceId: creds.deviceId,
      authToken: creds.authToken,
    );
  }

  /// Connect ke Host lalu kirim `auth` (reconnect).
  Future<void> connectAndAuth({
    required String host,
    required int port,
    required String deviceId,
    required String authToken,
  }) async {
    _sessionCreds = SavedCredentials(
      host: host,
      port: port,
      deviceId: deviceId,
      authToken: authToken,
    );
    state = state.copyWith(
      phase: ConnectionPhase.connecting,
      host: host,
      port: port,
    );
    try {
      await _ws.connect(host, port, autoReconnect: true);
      state = state.copyWith(phase: ConnectionPhase.authenticating);
      _ws.send(Outbound.auth(deviceId: deviceId, authToken: authToken));
    } catch (e) {
      state = state.copyWith(
        phase: ConnectionPhase.error,
        lastError: 'Gagal terhubung ke $host:$port — $e',
      );
    }
  }

  /// Pairing via QR (PRD FR-1 s.d. FR-4).
  /// [qrPayload] = JSON `{"host": "...", "port": ..., "token": "..."}`.
  Future<void> pairFromQr(String qrPayload) async {
    Map<String, dynamic> data;
    try {
      data = jsonDecode(qrPayload) as Map<String, dynamic>;
    } catch (e) {
      state = state.copyWith(
        phase: ConnectionPhase.error,
        lastError: 'QR tidak valid: $e',
      );
      return;
    }

    final host = data['host'] as String?;
    final port = (data['port'] as num?)?.toInt() ?? 48484;
    final token = data['token'] as String?;
    if (host == null || token == null) {
      state = state.copyWith(
        phase: ConnectionPhase.error,
        lastError: 'QR tidak berisi host/port/token',
      );
      return;
    }

    state = state.copyWith(phase: ConnectionPhase.connecting, host: host, port: port);
    try {
      await _ws.connect(host, port, autoReconnect: true);
      state = state.copyWith(phase: ConnectionPhase.authenticating);
      _ws.send(Outbound.pairRequest(
        pairToken: token,
        deviceName: await _deviceName(),
      ));
    } catch (e) {
      state = state.copyWith(
        phase: ConnectionPhase.error,
        lastError: 'Gagal terhubung ke $host:$port — $e',
      );
    }
  }

  Future<String> _deviceName() async {
    // Placeholder: nama device dari platform (bisa diganti UI di fase lanjut).
    return 'DashKey Device';
  }

  /// Kirim button_press (FR-16).
  void pressButton(String buttonId, String pageId) {
    if (state.phase != ConnectionPhase.authenticated) return;
    _ws.send(Outbound.buttonPress(buttonId: buttonId, pageId: pageId));
  }

  /// Pindah page aktif (FR-10).
  void switchPage(String pageId) {
    if (state.phase != ConnectionPhase.authenticated) return;
    _ws.send(Outbound.switchPage(pageId));
  }

  /// Pindah profile aktif (FR-10).
  void switchProfile(String profileId) {
    if (state.phase != ConnectionPhase.authenticated) return;
    _ws.send(Outbound.switchProfile(profileId));
  }

  void disconnect() {
    _sessionCreds = null;
    _ws.disconnect();
    state = state.copyWith(phase: ConnectionPhase.disconnected);
  }

  void _onMessage(ProtocolMessage msg) {
    switch (msg.type) {
      case Inbound.pairSuccess:
        final deviceId = msg.payload?['device_id'] as String?;
        final authToken = msg.payload?['auth_token'] as String?;
        if (deviceId != null && authToken != null) {
          final creds = SavedCredentials(
            host: state.host,
            port: state.port,
            deviceId: deviceId,
            authToken: authToken,
          );
          _sessionCreds = creds;
          ref.read(credentialStoreProvider).save(creds);
          // Langsung autentikasi di koneksi yang sama.
          _ws.send(Outbound.auth(deviceId: deviceId, authToken: authToken));
        }
      case Inbound.pairError:
        state = state.copyWith(
          phase: ConnectionPhase.error,
          lastError: 'Pairing gagal: ${msg.payload?['message']}',
        );
      case Inbound.authSuccess:
        state = state.copyWith(phase: ConnectionPhase.authenticated);
      case Inbound.authError:
        state = state.copyWith(
          phase: ConnectionPhase.error,
          lastError: 'Autentikasi gagal: ${msg.payload?['message']}',
        );
      case Inbound.configSync:
        final configJson = msg.payload?['profiles'];
        if (configJson is Map<String, dynamic>) {
          state = state.copyWith(config: ConfigData.fromJson(configJson));
        }
      case Inbound.actionResult:
        final buttonId = msg.payload?['button_id'] as String? ?? '';
        final success = msg.payload?['success'] as bool? ?? false;
        final message = msg.payload?['message'] as String?;
        state = state.copyWith(
          lastActionResult: (buttonId: buttonId, success: success, message: message),
        );
      case Inbound.statusUpdate:
        final buttonId = msg.payload?['button_id'] as String? ?? '';
        final status = msg.payload?['state'] as String? ?? '';
        final color = msg.payload?['color_override'] as String?;
        final next = Map<String, ({String state, String? colorOverride})>.from(
          state.buttonStates,
        );
        next[buttonId] = (state: status, colorOverride: color);
        state = state.copyWith(buttonStates: next);
      case Inbound.error:
        state = state.copyWith(lastError: 'Host: ${msg.payload?['message']}');
    }
  }
}
