import 'dart:convert';

import 'package:dashkey_controller/core/model/config.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('ConfigData', () {
    final raw = {
      'profiles': [
        {
          'profile_id': 'profile_streaming',
          'name': 'Streaming',
          'pages': ['page_main', 'page_obs'],
        }
      ],
      'pages': {
        'page_main': {
          'page_id': 'page_main',
          'name': 'Main',
          'grid_size': {'rows': 3, 'cols': 3},
          'buttons': ['btn_mute_mic'],
        }
      },
      'buttons': {
        'btn_mute_mic': {
          'button_id': 'btn_mute_mic',
          'label': 'Mute Mic',
          'color': '#FF3B30',
          'icon': 'mic.png',
          'actions': [
            {'action_type': 'obs_toggle_mute', 'target': 'Mic/Aux'},
          ],
        }
      },
      'active_profile': 'profile_streaming',
      'active_page': 'page_main',
    };

    test('parse config_sync payload dari Host', () {
      final config = ConfigData.fromJson(raw);
      expect(config.profiles.single.name, 'Streaming');
      expect(config.activePage, 'page_main');
      expect(config.currentPage?.gridSize.cols, 3);
      expect(config.currentButtons.single.label, 'Mute Mic');
      expect(config.currentButtons.single.actions.single.actionType,
          'obs_toggle_mute');
    });

    test('parseColor menangani hex valid & invalid', () {
      expect(ButtonDef.parseColor('#FF3B30'), 0xFFFF3B30);
      expect(ButtonDef.parseColor('zzz'), 0xFF1E88E5);
    });

    test('currentButtons mempertahankan urutan page.buttons', () {
      final raw2 = jsonDecode(jsonEncode(raw)) as Map<String, dynamic>;
      (raw2['pages'] as Map)['page_main'] = {
        'page_id': 'page_main',
        'name': 'Main',
        'grid_size': {'rows': 2, 'cols': 2},
        'buttons': ['btn_b', 'btn_a'],
      };
      (raw2['buttons'] as Map)['btn_a'] = {
        'button_id': 'btn_a',
        'label': 'A',
        'color': '#111111',
        'actions': [],
      };
      (raw2['buttons'] as Map)['btn_b'] = {
        'button_id': 'btn_b',
        'label': 'B',
        'color': '#222222',
        'actions': [],
      };
      final config = ConfigData.fromJson(raw2);
      expect(config.currentButtons.map((b) => b.label), ['B', 'A']);
    });
  });
}
