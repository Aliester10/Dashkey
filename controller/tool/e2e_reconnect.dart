// E2E test auto-reconnect: koneksi diputus paksa (host mati) → harus
// reconnect otomatis + re-auth saat host hidup kembali.
// Usage: dart run tool/e2e_reconnect.dart <device_id> <auth_token> [host] [port]
import 'dart:async';
import 'dart:io';

import 'package:dashkey_controller/core/network/ws_client.dart';
import 'package:dashkey_controller/core/protocol/messages.dart';

Future<void> main(List<String> args) async {
  if (args.length < 2) {
    print('usage: dart run tool/e2e_reconnect.dart <device_id> <auth_token>');
    return;
  }
  final deviceId = args[0];
  final authToken = args[1];
  final host = args.length > 2 ? args[2] : '127.0.0.1';
  final port = args.length > 3 ? int.parse(args[3]) : 48484;

  final conn = WsConnection();
  var authCount = 0;
  var connectedCount = 0;
  final allDone = Completer<void>();

  conn.statusStream.listen((s) {
    print('[status] $s');
    // Tiru perilaku ConnectionController: auth ulang setiap koneksi pulih.
    if (s == WsStatus.connected) {
      connectedCount++;
      if (connectedCount > 1) {
        conn.send(Outbound.auth(deviceId: deviceId, authToken: authToken));
        print('[send] auth ulang setelah reconnect');
      }
    }
  });
  conn.messages.listen((msg) {
    print('[recv] ${msg.type}');
    if (msg.type == Inbound.authSuccess) {
      authCount++;
      print('[recv] config_sync diharapkan menyusul');
    }
    if (msg.type == Inbound.configSync) {
      print('[recv] config_sync diterima (auth #$authCount)');
      if (authCount >= 2) {
        allDone.complete();
      }
    }
  });

  print('connect awal + auth...');
  await conn.connect(host, port, autoReconnect: true);
  conn.send(Outbound.auth(deviceId: deviceId, authToken: authToken));

  // Tunggu sampai auth kedua (setelah host dimatikan & dihidupkan lagi).
  await allDone.future.timeout(const Duration(seconds: 60));
  print('E2E RECONNECT OK (auth ulang setelah host pulih)');
  await conn.disconnect();
  exit(0);
}
