/// Bottom sheet editor satu aksi: pilih tipe aksi + isi parameternya.
library;

import 'package:flutter/material.dart';

/// Definisi tipe aksi yang didukung editor.
const actionTypeDefs = <String, ActionTypeDef>{
  'open_app': ActionTypeDef('Buka Aplikasi', 'target', 'Path/executable'),
  'close_app': ActionTypeDef('Tutup Aplikasi', 'target', 'nama proses, contoh: discord'),
  'open_url': ActionTypeDef('Buka URL', 'target', 'https://...'),
  'shell': ActionTypeDef('Jalankan Command', 'command', 'echo halo'),
  'hotkey': ActionTypeDef('Keyboard Shortcut', 'keys', 'ctrl,shift,s'),
  'play_sound': ActionTypeDef('Putar Suara', 'target', 'intro.mp3'),
  'media_control': ActionTypeDef('Kontrol Media', 'control', ''),
  'obs_switch_scene': ActionTypeDef('OBS: Pindah Scene', 'target', 'Scene Name'),
  'obs_toggle_mute': ActionTypeDef('OBS: Toggle Mute', 'target', 'Mic/Aux'),
  'obs_start_stream': ActionTypeDef('OBS: Start Stream', null, ''),
  'obs_stop_stream': ActionTypeDef('OBS: Stop Stream', null, ''),
  'obs_start_recording': ActionTypeDef('OBS: Start Recording', null, ''),
  'obs_stop_recording': ActionTypeDef('OBS: Stop Recording', null, ''),
};

/// Pilihan kontrol media untuk aksi `media_control`.
const mediaControls = [
  'play_pause', 'next', 'prev', 'stop',
  'volume_up', 'volume_down', 'mute',
];

class ActionTypeDef {
  const ActionTypeDef(this.label, this.field, this.hint);

  final String label;
  final String? field;
  final String hint;
}

/// Label detail pendek sebuah aksi.
String? actionLabel(String type, Map<String, dynamic> action) {
  final def = actionTypeDefs[type];
  if (def == null || def.field == null) return null;
  final value = action[def.field];
  if (value is List) return value.join(' + ');
  final text = value as String? ?? '';
  return text.isEmpty ? null : text;
}

/// Ikon per tipe aksi.
IconData actionIcon(String type) => switch (type) {
      'open_app' => Icons.apps,
      'close_app' => Icons.close,
      'open_url' => Icons.public,
      'shell' => Icons.terminal,
      'hotkey' => Icons.keyboard,
      'play_sound' => Icons.music_note,
      'media_control' => Icons.play_circle_outline,
      'obs_switch_scene' => Icons.switch_video,
      'obs_toggle_mute' => Icons.mic_off,
      _ => Icons.live_tv,
    };

class ActionEditorSheet extends StatefulWidget {
  const ActionEditorSheet({super.key, this.initial});

  final Map<String, dynamic>? initial;

  @override
  State<ActionEditorSheet> createState() => _ActionEditorSheetState();
}

class _ActionEditorSheetState extends State<ActionEditorSheet> {
  late String _type;
  final _textController = TextEditingController();
  String _mediaControl = mediaControls.first;
  bool _force = false;

  @override
  void initState() {
    super.initState();
    _type = (widget.initial?['action_type'] as String?) ?? 'open_app';
    final def = actionTypeDefs[_type];
    if (def != null && def.field != null) {
      final value = widget.initial?[def.field];
      if (value is String) _textController.text = value;
      if (value is List) _textController.text = value.join(',');
      if (def.field == 'control') _mediaControl = value as String? ?? mediaControls.first;
    }
    _force = widget.initial?['force'] as bool? ?? false;
  }

  @override
  void dispose() {
    _textController.dispose();
    super.dispose();
  }

  Map<String, dynamic> _buildAction() {
    final def = actionTypeDefs[_type]!;
    final action = <String, dynamic>{'action_type': _type};
    switch (def.field) {
      case 'target' || 'command':
        action[def.field!] = _textController.text.trim();
      case 'keys':
        action['keys'] = _textController.text
            .split(',')
            .map((s) => s.trim().toLowerCase())
            .where((s) => s.isNotEmpty)
            .toList();
      case 'control':
        action['control'] = _mediaControl;
      default:
        break;
    }
    if (_type == 'close_app' && _force) {
      action['force'] = true;
    }
    return action;
  }

  @override
  Widget build(BuildContext context) {
    final def = actionTypeDefs[_type]!;
    return Padding(
      padding: EdgeInsets.only(
        left: 16,
        right: 16,
        top: 16,
        bottom: MediaQuery.of(context).viewInsets.bottom + 16,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('Tipe Aksi', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          DropdownButtonFormField<String>(
            initialValue: _type,
            items: actionTypeDefs.entries
                .map(
                  (e) => DropdownMenuItem(
                    value: e.key,
                    child: Text(e.value.label),
                  ),
                )
                .toList(),
            onChanged: (v) => setState(() => _type = v ?? 'open_app'),
          ),
          const SizedBox(height: 12),
          if (def.field == 'control')
            DropdownButtonFormField<String>(
              initialValue: _mediaControl,
              items: mediaControls
                  .map((c) => DropdownMenuItem(value: c, child: Text(c)))
                  .toList(),
              onChanged: (v) =>
                  setState(() => _mediaControl = v ?? mediaControls.first),
            )
          else if (def.field != null)
            TextField(
              controller: _textController,
              decoration: InputDecoration(
                labelText: def.field == 'keys'
                    ? 'Keys (pisahkan dengan koma)'
                    : def.hint,
                border: const OutlineInputBorder(),
              ),
            ),
          if (_type == 'close_app') ...[
            const SizedBox(height: 8),
            SwitchListTile(
              contentPadding: EdgeInsets.zero,
              title: const Text('Force close'),
              subtitle: const Text(
                'Paksa tutup (data yang belum disimpan bisa hilang)',
              ),
              value: _force,
              onChanged: (v) => setState(() => _force = v),
            ),
          ],
          const SizedBox(height: 16),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(_buildAction()),
            child: const Text('Simpan Aksi'),
          ),
        ],
      ),
    );
  }
}
