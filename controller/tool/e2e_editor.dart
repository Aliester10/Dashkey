// E2E editor: pair+auth → modifikasi config (tambah tombol) → save_config
// → config_saved + config_sync broadcast → verifikasi.
// Usage: dart run tool/e2e_editor.dart <pair_token>
import 'dart:async';
import 'dart:convert';

import 'package:dashkey_controller/core/model/config.dart';
import 'package:dashkey_controller/core/network/ws_client.dart';
import 'package:dashkey_controller/core/protocol/messages.dart';

Future<void> main(List<String> args) async {
  if (args.isEmpty) {
    print('usage: dart run tool/e2e_editor.dart <pair_token>');
    return;
  }
  final token = args[0];
  final conn = WsConnection();
  final done = Completer<void>();
  ConfigData? synced;

  conn.messages.listen((msg) {
    print('[recv] ${msg.type}');
    switch (msg.type) {
      case Inbound.pairSuccess:
        final deviceId = msg.payload?['device_id'] as String;
        final authToken = msg.payload?['auth_token'] as String;
        conn.send(Outbound.auth(deviceId: deviceId, authToken: authToken));
      case Inbound.authSuccess:
        break; // tidak perlu aksi; config_sync menyusul
      case Inbound.configSync:
        final cfg = msg.payload?['profiles'] as Map<String, dynamic>;
        synced = ConfigData.fromJson(cfg);
        if (done.isCompleted) return;
        if (synced!.buttons.values.any((b) => b.label == 'Tombol dari Tool E2E')) {
          print('[ok] tombol baru terlihat di config_sync broadcast');
          done.complete();
        }
      case Inbound.configSaved:
        print('[recv] config_saved: ${msg.payload?['message']}');
      default:
        break;
    }
  });

  await conn.connect('127.0.0.1', 48484);
  conn.send(Outbound.pairRequest(pairToken: token, deviceName: 'E2E Editor Tool'));

  // Tunggu config_sync awal.
  final firstSync = await conn.messages
      .firstWhere((m) => m.type == Inbound.configSync)
      .asStream()
      .first;
  final config = ConfigData.fromJson(
    firstSync.payload?['profiles'] as Map<String, dynamic>,
  );

  // Modifikasi: tambah tombol baru ke page aktif.
  final draft = jsonDecode(jsonEncode(config.toJson())) as Map<String, dynamic>;
  final buttonId = 'btn_e2e_${DateTime.now().millisecondsSinceEpoch}';
  (draft['buttons'] as Map<String, dynamic>)[buttonId] = {
    'button_id': buttonId,
    'label': 'Tombol dari Tool E2E',
    'color': '#00AA00',
    'actions': [
      {'action_type': 'open_url', 'target': 'https://example.com'},
    ],
  };
  final activePage = draft['active_page'] as String;
  final page = (draft['pages'] as Map<String, dynamic>)[activePage]
      as Map<String, dynamic>;
  (page['buttons'] as List).add(buttonId);

  print('--- kirim save_config ---');
  conn.send(ProtocolMessage(type: 'save_config', payload: {'config': draft}));

  await done.future.timeout(const Duration(seconds: 10));
  await conn.disconnect();
  print('E2E EDITOR OK');
}
