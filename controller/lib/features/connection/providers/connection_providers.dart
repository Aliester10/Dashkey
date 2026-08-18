/// Provider state koneksi WebSocket (riverpod).
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/network/ws_client.dart';
import '../../../core/protocol/messages.dart';

/// Instance koneksi WebSocket global.
final wsConnectionProvider = Provider<WsConnection>((ref) {
  final conn = WsConnection();
  ref.onDispose(conn.dispose);
  return conn;
});

/// Status koneksi sebagai state yang bisa di-watch UI.
final wsStatusProvider = StreamProvider<WsStatus>((ref) {
  final conn = ref.watch(wsConnectionProvider);
  return conn.statusStream;
});

/// Stream pesan masuk dari Host (echo reply, status update, dsb).
final wsMessagesProvider = StreamProvider<ProtocolMessage>((ref) {
  final conn = ref.watch(wsConnectionProvider);
  return conn.messages;
});
