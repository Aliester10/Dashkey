/// Wrapper WebSocket client (web_socket_channel).
///
/// Mendukung auto-reconnect dengan exponential backoff (PRD NFR Reliabilitas):
/// saat koneksi putus tak sengaja (Wi-Fi drop, PC sleep), client otomatis
/// mencoba koneksi ulang memakai host/port terakhir.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:math';

import 'package:web_socket_channel/web_socket_channel.dart';

import '../protocol/messages.dart';

/// Status koneksi saat ini.
enum WsStatus { disconnected, connecting, connected, error }

/// Koneksi WebSocket ke Host DashKey.
class WsConnection {
  WsConnection();

  WebSocketChannel? _channel;
  StreamSubscription<dynamic>? _sub;

  final _statusController = StreamController<WsStatus>.broadcast();
  final _messagesController = StreamController<ProtocolMessage>.broadcast();

  WsStatus status = WsStatus.disconnected;

  bool _autoReconnect = false;
  bool _manuallyClosed = false;
  String? _host;
  int? _port;
  int _reconnectAttempt = 0;
  Timer? _reconnectTimer;

  /// Stream status koneksi.
  Stream<WsStatus> get statusStream => _statusController.stream;

  /// Stream pesan masuk dari Host.
  Stream<ProtocolMessage> get messages => _messagesController.stream;

  /// Connect ke Host pada [host] dan [port].
  /// [autoReconnect]: coba koneksi ulang otomatis bila koneksi terputus.
  Future<void> connect(
    String host,
    int port, {
    bool autoReconnect = false,
  }) async {
    await disconnect();

    _host = host;
    _port = port;
    _autoReconnect = autoReconnect;
    _manuallyClosed = false;

    _setStatus(WsStatus.connecting);

    final uri = Uri.parse('ws://$host:$port');
    try {
      final channel = WebSocketChannel.connect(uri);
      await channel.ready;
      _channel = channel;
      _reconnectAttempt = 0;
      _sub = channel.stream.listen(
        _onData,
        onError: (Object e) {
          _setStatus(WsStatus.error);
          _onConnectionLost();
        },
        onDone: _onConnectionLost,
        cancelOnError: false,
      );
      _setStatus(WsStatus.connected);
    } catch (e) {
      _setStatus(WsStatus.error);
      rethrow;
    }
  }

  /// Koneksi terputus (tidak sengaja) → jadwalkan reconnect.
  void _onConnectionLost() {
    _teardown();
    if (_manuallyClosed) return;
    _setStatus(WsStatus.disconnected);
    if (_autoReconnect && _host != null && _port != null) {
      _scheduleReconnect();
    }
  }

  /// Reconnect dengan exponential backoff: 1s, 2s, 4s, 8s, cap 10s.
  void _scheduleReconnect() {
    final attempt = _reconnectAttempt++;
    final seconds = min(10, 1 << min(attempt, 4));
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(Duration(seconds: seconds), () async {
      if (_manuallyClosed) return;
      final host = _host!;
      final port = _port!;
      try {
        await connect(host, port, autoReconnect: true);
      } catch (_) {
        _onConnectionLost();
      }
    });
  }

  void _onData(dynamic data) {
    if (data is String) {
      try {
        final json = jsonDecode(data);
        if (json is Map<String, dynamic>) {
          _messagesController.add(ProtocolMessage.fromJson(json));
        }
      } catch (_) {
        // Abaikan frame yang bukan JSON valid.
      }
    }
  }

  /// Kirim pesan ke Host.
  void send(ProtocolMessage message) {
    final channel = _channel;
    if (channel == null || status != WsStatus.connected) {
      throw StateError('Tidak terhubung ke Host');
    }
    channel.sink.add(message.encode());
  }

  Future<void> disconnect() async {
    _manuallyClosed = true;
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
    _teardown();
  }

  void _teardown() {
    _sub?.cancel();
    _sub = null;
    _channel?.sink.close();
    _channel = null;
  }

  void _setStatus(WsStatus next) {
    status = next;
    if (!_statusController.isClosed) _statusController.add(next);
  }

  Future<void> dispose() async {
    await disconnect();
    await _statusController.close();
    await _messagesController.close();
  }
}
