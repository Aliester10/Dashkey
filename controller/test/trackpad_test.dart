import 'dart:convert';

import 'package:dashkey_controller/core/model/config.dart';
import 'package:dashkey_controller/core/protocol/messages.dart';
import 'package:dashkey_controller/features/grid/widgets/button_icon.dart';
import 'package:dashkey_controller/features/trackpad/trackpad_area.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('PRD2 Trackpad', () {
    test('mouse_move encode sesuai PRD2 6.1', () {
      final msg = Outbound.mouseMove(dx: 4, dy: -2);
      expect(msg.encode(), '{"type":"mouse_move","payload":{"dx":4,"dy":-2}}');
    });

    test('mouse_click encode sesuai PRD2 6.2', () {
      final msg = Outbound.mouseClick('right');
      expect(msg.encode(), '{"type":"mouse_click","payload":{"button":"right"}}');
    });

    test('mouse_scroll encode sesuai PRD2 6.3', () {
      final msg = Outbound.mouseScroll(-3);
      expect(msg.encode(), '{"type":"mouse_scroll","payload":{"dy":-3}}');
    });

    test('mouse_down/up untuk drag sesuai PRD2 6.4', () {
      expect(
        Outbound.mouseDown('left').encode(),
        '{"type":"mouse_down","payload":{"button":"left"}}',
      );
      expect(
        Outbound.mouseUp('left').encode(),
        '{"type":"mouse_up","payload":{"button":"left"}}',
      );
    });

    test('PageDef parse page_type trackpad', () {
      final page = PageDef.fromJson({
        'page_id': 'p1',
        'name': 'Trackpad',
        'grid_size': {'rows': 4, 'cols': 4},
        'buttons': <String>[],
        'page_type': 'trackpad',
      });
      expect(page.isTrackpad, true);
      expect(page.toJson()['page_type'], 'trackpad');
    });

    test('PageDef default page_type = buttons (config lama)', () {
      final page = PageDef.fromJson({
        'page_id': 'p1',
        'name': 'Main',
        'grid_size': {'rows': 3, 'cols': 3},
        'buttons': <String>[],
      });
      expect(page.isTrackpad, false);
      expect(page.toJson()['page_type'], 'buttons');
    });

    test('round-trip config dengan page_type', () {
      final raw = {
        'profiles': [
          {'profile_id': 'p', 'name': 'P', 'pages': ['t']}
        ],
        'pages': {
          't': {
            'page_id': 't',
            'name': 'Touch',
            'grid_size': {'rows': 2, 'cols': 2},
            'buttons': <String>[],
            'page_type': 'trackpad',
          }
        },
        'buttons': <String, dynamic>{},
        'active_profile': 'p',
        'active_page': 't',
      };
      final back = ConfigData.fromJson(raw).toJson();
      expect(jsonEncode(back), jsonEncode(raw));
    });

    test('iconForKey semantic keys', () {
      const actions = <ActionDef>[];
      expect(iconForKey('app', actions), Icons.apps);
      expect(iconForKey('music', actions), Icons.music_note);
      expect(iconForKey('obs', actions), Icons.live_tv);
      expect(iconForKey('game', actions), Icons.sports_esports);
      expect(iconForKey('file:///x.png', actions), isNull);
      expect(iconForKey('unknown-key', actions), Icons.touch_app);
    });

    test('iconForKey fallback dari aksi pertama', () {
      const actions = [ActionDef(actionType: 'open_url')];
      expect(iconForKey(null, actions), Icons.public);
    });

    test('sensitivitas slider dalam rentang wajar', () {
      final factor = TrackpadSensitivity.fromSlider(5);
      expect(factor, greaterThan(0));
      expect(factor, lessThan(4.0));
      expect(TrackpadSensitivity.fromSlider(10), greaterThan(TrackpadSensitivity.fromSlider(1)));
    });
  });
}
