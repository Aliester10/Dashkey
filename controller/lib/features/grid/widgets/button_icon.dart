/// Pemetaan ikon tombol (semantic key dari desktop/Host) ke Material icon.
/// Sinkronisasi style: apa pun key yang disimpan Host, HP menampilkan
/// ikon yang sama (fallback: berdasarkan jenis aksi pertama).
/// Return null bila tombol memakai gambar file:// (desktop) — tile
/// menampilkan huruf pertama label sebagai gantinya (sama seperti desktop).
library;

import 'package:flutter/material.dart';

import '../../../core/model/config.dart';

/// Map semantic key (dipakai desktop GUI & importer SFX) → Material icon.
IconData? iconForKey(String? key, List<ActionDef> actions) {
  if (key == null) {
    return _iconFromActions(actions);
  }
  if (key.startsWith('file://')) {
    // Gambar lokal PC tidak bisa dirender di HP — fallback huruf pertama.
    return null;
  }
  switch (key) {
    case 'lightning' || 'bolt':
      return Icons.bolt;
    case 'app' || 'apps':
      return Icons.apps;
    case 'url' || 'web':
      return Icons.public;
    case 'hotkey' || 'keyboard':
      return Icons.keyboard;
    case 'music' || 'sfx' || 'sound':
      return Icons.music_note;
    case 'media':
      return Icons.play_circle_outline;
    case 'mic' || 'mute':
      return Icons.mic_off;
    case 'game':
      return Icons.sports_esports;
    case 'terminal' || 'shell' || 'code':
      return Icons.terminal;
    case 'obs' || 'stream' || 'video':
      return Icons.live_tv;
    case 'folder':
      return Icons.folder_open;
    case 'star':
      return Icons.star;
    case 'heart':
      return Icons.favorite;
    case 'camera':
      return Icons.videocam;
    case 'chat':
      return Icons.chat;
    case 'rocket':
      return Icons.rocket_launch;
    case 'clock':
      return Icons.access_time;
    case 'mail':
      return Icons.mail;
    default:
      return _iconFromActions(actions);
  }
}

/// Fallback: ikon dari aksi pertama tombol.
IconData _iconFromActions(List<ActionDef> actions) {
  for (final action in actions) {
    switch (action.actionType) {
      case 'open_app':
        return Icons.apps;
      case 'open_url':
        return Icons.public;
      case 'hotkey':
        return Icons.keyboard;
      case 'shell':
        return Icons.terminal;
      case 'play_sound':
        return Icons.music_note;
      case 'media_control':
        return Icons.play_circle_outline;
      case 'obs_switch_scene':
      case 'obs_toggle_mute':
      case 'obs_start_stream':
      case 'obs_stop_stream':
      case 'obs_start_recording':
      case 'obs_stop_recording':
        return Icons.live_tv;
    }
  }
  return Icons.touch_app;
}
