import 'dart:convert';

import 'package:dashkey_controller/core/protocol/messages.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('ProtocolMessage', () {
    test('encode echo sesuai format PRD', () {
      final msg = Outbound.echo('halo');
      expect(msg.encode(), '{"type":"echo","payload":{"text":"halo"}}');
    });

    test('encode button_press sesuai format PRD 8.4', () {
      final msg = Outbound.buttonPress(
        buttonId: 'btn_airhorn',
        pageId: 'page_soundboard',
      );
      expect(
        msg.encode(),
        '{"type":"button_press","payload":{"button_id":"btn_airhorn","page_id":"page_soundboard"}}',
      );
    });

    test('encode ping dengan payload null (unit variant di Host)', () {
      final msg = Outbound.ping();
      final decoded = jsonDecode(msg.encode()) as Map<String, dynamic>;
      expect(decoded['type'], 'ping');
      expect(decoded['payload'], isNull);
    });

    test('decode status_update dari Host', () {
      final raw =
          '{"type":"status_update","payload":{"button_id":"btn_mute_mic","state":"active","color_override":"#FF3B30"}}';
      final msg = ProtocolMessage.fromJson(
        jsonDecode(raw) as Map<String, dynamic>,
      );
      expect(msg.type, Inbound.statusUpdate);
      expect(msg.payload?['button_id'], 'btn_mute_mic');
      expect(msg.payload?['color_override'], '#FF3B30');
    });
  });
}
