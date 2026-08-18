// E2E test full flow (Dart): pairing -> auth -> config_sync -> button_press.
// Jalankan host mode pair dulu, lalu:
//   dart run tool/e2e_full.dart <pair_token>
import 'dart:async';
import 'dart:convert';

import 'package:dashkey_controller/core/network/ws_client.dart';
import 'package:dashkey_controller/core/protocol/messages.dart';

Future<void> main(List<String> args) async {
  final token = args.isNotEmpty ? args[0] : '';
  final host = args.length > 1 ? args[1] : '127.0.0.1';
  final port = args.length > 2 ? int.parse(args[2]) : 48484;
  if (token.isEmpty) {
    print('usage: dart run tool/e2e_full.dart <pair_token> [host] [port]');
    return;
  }

  final conn = WsConnection();
  final done = Completer<void>();
  final expected = [
    'pair_success',
    'auth_success',
    'config_sync',
    'action_result',
  ];
  final received = <String>[];

  conn.messages.listen((msg) {
    final payload = jsonEncode(msg.payload);
    final preview =
        payload.length > 150 ? payload.substring(0, 150) : payload;
    print('[recv] ${msg.type} payload=$preview');
    received.add(msg.type);
    if (expected.every(received.contains)) done.complete();
  });

  await conn.connect(host, port);
  print('--- pairing ---');
  conn.send(Outbound.pairRequest(pairToken: token, deviceName: 'Dart E2E Device'));

  // Setelah pair_success, kirim auth dengan kredensial dari balasan.
  final pairSuccess = conn.messages
      .firstWhere((m) => m.type == Inbound.pairSuccess)
      .asStream()
      .first;
  final pairMsg = await pairSuccess;
  final deviceId = pairMsg.payload?['device_id'];
  final authToken = pairMsg.payload?['auth_token'];
  print('--- auth dengan $deviceId ---');
  conn.send(Outbound.auth(deviceId: deviceId!, authToken: authToken!));

  // Setelah config_sync, tekan tombol pertama dari config.
  final syncMsg = await conn.messages
      .firstWhere((m) => m.type == Inbound.configSync)
      .asStream()
      .first;
  final config = syncMsg.payload?['profiles'] as Map<String, dynamic>;
  final pages = config['pages'] as Map<String, dynamic>;
  final activePage = config['active_page'] as String;
  final firstButton = ((pages[activePage] as Map)['buttons'] as List).first;
  print('--- button_press $firstButton ---');
  conn.send(Outbound.buttonPress(buttonId: firstButton, pageId: activePage));

  await done.future.timeout(const Duration(seconds: 10));
  await conn.disconnect();
  print('E2E FULL OK: ${received.join(' -> ')}');
}
