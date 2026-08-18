/// Area trackpad (PRD2) — gesture 1/2 jari dikonversi ke pesan mouse,
/// delta diakumulasi lalu di-flush per frame Ticker (60Hz) agar tidak
/// membanjiri Host (NFR throughput PRD2 §4).
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';

import '../../core/protocol/messages.dart';

/// Faktor pengali sensitivitas kursor.
class TrackpadSensitivity {
  static const min = 0.2;
  static const max = 4.0;

  /// Konversi dari slider 1..10.
  static double fromSlider(int value) => min + (value - 1) / 9 * (max - min);
}

/// Threshold jarak (logical px) sebelum gerakan dianggap drag,
/// bukan tap (FR-T9: default ±5px).
const _tapThreshold = 5.0;

/// Durasi tekan-tahan sebelum dianggap drag (FR-T6).
const _holdDuration = Duration(milliseconds: 280);

class TrackpadArea extends StatefulWidget {
  const TrackpadArea({
    super.key,
    required this.onMessage,
    required this.sensitivity,
  });

  /// Kirim satu pesan mouse ke Host.
  final void Function(ProtocolMessage message) onMessage;

  /// Faktor pengali dx/dy (dari Settings).
  final double sensitivity;

  @override
  State<TrackpadArea> createState() => _TrackpadAreaState();
}

class _TrackpadAreaState extends State<TrackpadArea>
    with SingleTickerProviderStateMixin {
  late final Ticker _ticker;

  // Akumulasi delta per frame.
  double _accDx = 0;
  double _accDy = 0;
  double _scrollDy = 0;

  // State multi-touch.
  final Map<int, Offset> _pointers = {};
  double _totalDistance = 0;
  Timer? _holdTimer;
  bool _isDragging = false;
  bool _twoFingerGesture = false;
  bool _twoFingerMoved = false;

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_onTick)..start();
  }

  @override
  void dispose() {
    _ticker.dispose();
    _holdTimer?.cancel();
    _endDrag(forceRelease: true);
    super.dispose();
  }

  /// Flush akumulasi setiap tick (~60Hz).
  void _onTick(Duration elapsed) {
    if (_isDragging || _pointers.length == 1) {
      final dx = _accDx.round();
      final dy = _accDy.round();
      if (dx != 0 || dy != 0) {
        widget.onMessage(Outbound.mouseMove(dx: dx, dy: dy));
        _accDx = 0;
        _accDy = 0;
      }
    }
    if (_pointers.length == 2 && _scrollDy.abs() >= 1) {
      final dy = _scrollDy.round();
      widget.onMessage(Outbound.mouseScroll(dy));
      _scrollDy = 0;
    }
  }

  // ── Pointer events ─────────────────────────────────────────────────────

  void _onPointerDown(PointerDownEvent event) {
    _pointers[event.pointer] = event.position;

    if (_pointers.length == 1) {
      _totalDistance = 0;
      _isDragging = false;
      _holdTimer?.cancel();
      // Tekan-tahan → mulai drag (FR-T6).
      _holdTimer = Timer(_holdDuration, () {
        if (!mounted || _pointers.length != 1) return;
        _isDragging = true;
        widget.onMessage(Outbound.mouseDown('left'));
      });
    } else if (_pointers.length == 2) {
      // Jari kedua muncul: batalkan drag-hold, siapkan gestur 2 jari.
      _holdTimer?.cancel();
      if (_isDragging) {
        _isDragging = false;
        widget.onMessage(Outbound.mouseUp('left'));
      }
      _twoFingerGesture = true;
      _twoFingerMoved = false;
    }
  }

  void _onPointerMove(PointerMoveEvent event) {
    final previous = _pointers[event.pointer];
    if (previous == null) return;
    final delta = event.position - previous;
    _pointers[event.pointer] = event.position;

    if (_pointers.length == 1) {
      _totalDistance += delta.distance;
      if (_holdTimer != null && _totalDistance > _tapThreshold) {
        // Bergerak sebelum hold selesai → bukan tap, bukan drag-hold.
        _holdTimer?.cancel();
        _holdTimer = null;
      }
      if (_isDragging || _totalDistance > _tapThreshold) {
        _accDx += delta.dx * widget.sensitivity;
        _accDy += delta.dy * widget.sensitivity;
      }
    } else if (_pointers.length == 2) {
      // Scroll vertikal 2 jari (FR-T5).
      _twoFingerMoved = true;
      final dy = delta.dy;
      // Invers: geser ke atas (dy negatif) = scroll ke atas? Konvensi
      // natural scroll: konten ikut jari → dy positif scroll ke bawah.
      _scrollDy += dy * widget.sensitivity * 0.5;
      // Jika dua jari bergerak, gerakan kursor tidak dikirim.
    }
  }

  void _onPointerUp(PointerUpEvent event) => _handlePointerEnd(event.pointer);

  void _onPointerCancel(PointerCancelEvent event) => _handlePointerEnd(event.pointer);

  void _handlePointerEnd(int pointer) {
    _pointers.remove(pointer);

    if (_pointers.isEmpty) {
      _holdTimer?.cancel();
      if (_twoFingerGesture) {
        // FR-T4: tap 2 jari tanpa geser → klik kanan.
        if (!_twoFingerMoved && !_isDragging) {
          widget.onMessage(Outbound.mouseClick('right'));
        }
        _twoFingerGesture = false;
        _twoFingerMoved = false;
        return;
      }
      if (_isDragging) {
        _isDragging = false;
        widget.onMessage(Outbound.mouseUp('left'));
      } else if (_totalDistance <= _tapThreshold) {
        // Tap singkat tanpa geser → klik kiri (FR-T3).
        widget.onMessage(Outbound.mouseClick('left'));
      }
    } else if (_pointers.length == 1) {
      // Satu jari terangkat dari gestur 2 jari — pertahankan status.
      _twoFingerMoved = true; // jangan jadikan klik saat tersisa 1 jari
    }
  }

  void _endDrag({bool forceRelease = false}) {
    if (forceRelease && _isDragging) {
      _isDragging = false;
      widget.onMessage(Outbound.mouseUp('left'));
    }
  }

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Container(
      decoration: BoxDecoration(
        color: scheme.surfaceContainerHighest.withValues(alpha: 0.35),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(
          color: scheme.primary.withValues(alpha: 0.4),
          width: 1.5,
        ),
      ),
      child: Listener(
        behavior: HitTestBehavior.opaque,
        onPointerDown: _onPointerDown,
        onPointerMove: _onPointerMove,
        onPointerUp: _onPointerUp,
        onPointerCancel: _onPointerCancel,
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.touch_app, size: 48, color: scheme.primary),
              const SizedBox(height: 8),
              Text(
                '1 jari: gerak / tap kiri\n'
                '2 jari: scroll / tap kanan\n'
                'tekan-tahan: drag',
                textAlign: TextAlign.center,
                style: TextStyle(color: scheme.onSurfaceVariant, fontSize: 12),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
