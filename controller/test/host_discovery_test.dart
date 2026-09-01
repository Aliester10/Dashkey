import 'dart:convert';

import 'package:dashkey_controller/features/pairing/services/host_discovery.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('HostDiscovery', () {
    test('parses dashkey_hello dari Host', () {
      final json = jsonDecode(
        '{"type":"dashkey_hello","host":"192.168.1.5","port":48484,'
        '"host_name":"PC-Fabert","version":"0.1.0","pair_token":"tok-123"}',
      ) as Map<String, dynamic>;
      final host = DiscoveredHost.fromJson(json);
      expect(host.host, '192.168.1.5');
      expect(host.port, 48484);
      expect(host.hostName, 'PC-Fabert');
      expect(host.version, '0.1.0');
      expect(host.pairToken, 'tok-123');
      expect(host.isValid, true);
    });

    test('hello tanpa host/pair_token dianggap tidak valid', () {
      final host = DiscoveredHost.fromJson({'type': 'dashkey_hello'});
      expect(host.isValid, false);
      final host2 = DiscoveredHost.fromJson({
        'type': 'dashkey_hello',
        'host': '192.168.1.5',
        'port': 48484,
      });
      expect(host2.isValid, false);
    });

    test('port default 48484 bila tidak ada', () {
      final host = DiscoveredHost.fromJson({
        'type': 'dashkey_hello',
        'host': '10.0.0.2',
        'pair_token': 'x',
      });
      expect(host.port, 48484);
      expect(host.isValid, true);
    });
  });
}