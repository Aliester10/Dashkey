import 'dart:convert';

import 'package:dashkey_controller/core/model/config.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('ConfigData round-trip (toJson/fromJson)', () {
    final raw = {
      'profiles': [
        {
          'profile_id': 'profile_default',
          'name': 'Default',
          'pages': ['page_main'],
        }
      ],
      'pages': {
        'page_main': {
          'page_id': 'page_main',
          'name': 'Main',
          'grid_size': {'rows': 4, 'cols': 4},
          'buttons': ['btn_a'],
          'page_type': 'buttons',
        }
      },
      'buttons': {
        'btn_a': {
          'button_id': 'btn_a',
          'label': 'A',
          'color': '#1E88E5',
          'icon': 'a.png',
          'actions': [
            {'action_type': 'open_url', 'target': 'https://example.com'},
            {'action_type': 'hotkey', 'keys': ['ctrl', 'a']},
            {'action_type': 'media_control', 'control': 'play_pause'},
            {'action_type': 'obs_start_stream'},
          ],
        }
      },
      'active_profile': 'profile_default',
      'active_page': 'page_main',
      'obs': {'host': '127.0.0.1', 'port': 4455, 'password': 'x'},
    };

    test('toJson menghasilkan struktur identik dengan input', () {
      final config = ConfigData.fromJson(raw);
      final back = config.toJson();
      expect(jsonEncode(back), jsonEncode(raw));
    });

    test('icon null dihilangkan (host terima Option None)', () {
      final raw2 = jsonDecode(jsonEncode(raw)) as Map<String, dynamic>;
      (raw2['buttons'] as Map)['btn_a'] = {
        'button_id': 'btn_a',
        'label': 'A',
        'color': '#111111',
        'actions': <Object>[],
      };
      final back = ConfigData.fromJson(raw2).toJson();
      expect((back['buttons'] as Map)['btn_a'] is Map, true);
      expect(
        ((back['buttons'] as Map)['btn_a'] as Map).containsKey('icon'),
        false,
      );
    });

    test('action summary untuk hotkey', () {
      final action = ActionDef(actionType: 'hotkey', keys: ['ctrl', 'a']);
      expect(action.summary, 'hotkey: ctrl+a');
      final empty = ActionDef(actionType: 'obs_start_stream');
      expect(empty.summary, 'obs_start_stream');
    });

    test('icon_data (base64) dari Host di-parse', () {
      final button = ButtonDef.fromJson({
        'button_id': 'b1',
        'label': 'Discord',
        'color': '#111111',
        'icon': 'file:///usr/share/pixmaps/discord.png',
        'icon_data': 'aGVsbG8=',
        'actions': <Object>[],
      });
      expect(button.iconData, 'aGVsbG8=');
      final back = button.toJson();
      expect(back['icon_data'], 'aGVsbG8=');
    });
  });
}
