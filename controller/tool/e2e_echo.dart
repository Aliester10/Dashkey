// E2E test: connect ke Host (127.0.0.1:48484), kirim echo & ping.
// Jalankan: dart run tool/e2e_echo.dart
import 'dart:async';

import 'package:dashkey_controller/core/network/ws_client.dart';
import 'package:dashkey_controller/core/protocol/messages.dart';

Future<void> main(List<String> args) async {
  final host = args.isNotEmpty ? args[0] : '127.0.0.1';
  final port = args.length > 1 ? int.parse(args[1]) : 48484;

  final conn = WsConnection();
  final done = Completer<void>();

  conn.statusStream.listen((s) => print('[status] $s'));
  conn.messages.listen((msg) {
    print('[recv] ${msg.type} payload=${msg.payload}');
    if (msg.type == Inbound.echoReply) done.complete();
  });

  print('connecting to $host:$port ...');
  await conn.connect(host, port);
  print('connected: ${conn.status}');

  conn.send(Outbound.echo('halo dari dart e2e'));
  conn.send(Outbound.ping());

  await done.future.timeout(const Duration(seconds: 5));
  await conn.disconnect();
  print('E2E OK');
}
