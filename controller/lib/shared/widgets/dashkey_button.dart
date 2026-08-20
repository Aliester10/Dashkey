/// DashKeyButton — tombol reusable dengan Digital Tactile Feedback (prdbutton.md)
/// dan gesture Controller: tap (utama), double tap (sekunder), long press (close).
///
/// Gesture:
/// - Tap 1×        → `onPressed` (aksi utama)
/// - Double tap 2× → `onDoubleTap` (aksi sekunder; HANYA jika disediakan,
///   sehingga tombol tanpa aksi sekunder tetap responsif tanpa delay)
/// - Long press    → `onLongPress` (default: close app), dengan ring progres
///   + haptic medium saat siap mengeksekusi
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/feedback/feedback_config.dart';
import '../../core/feedback/feedback_engine.dart';
import '../theme/app_colors.dart';
import '../theme/app_shadows.dart';

class DashKeyButton extends ConsumerStatefulWidget {
  const DashKeyButton({
    super.key,
    required this.child,
    this.onPressed,
    this.onDoubleTap,
    this.onLongPress,
    this.longPressDuration = const Duration(milliseconds: 600),
    this.disabled = false,
    this.active = false,
    this.accentColor,
    this.haptic = DashHaptic.light,
    this.borderRadius = 16,
    this.padding = const EdgeInsets.all(14),
    this.surfaceColor,
  });

  final Widget child;
  final VoidCallback? onPressed;

  /// Aksi sekunder (double tap). Jika null, tombol tidak menunggu double tap.
  final VoidCallback? onDoubleTap;

  /// Aksi long press (mis. close app). Jika null, tombol menahan tanpa aksi.
  final VoidCallback? onLongPress;

  /// Durasi tahan sebelum long press ter-trigger.
  final Duration longPressDuration;

  /// State disabled (PRD §2.4): opacity turun, tanpa interaksi.
  final bool disabled;

  /// State active persistent (PRD §2.3): border accent + glow.
  final bool active;

  final Color? accentColor;

  /// Jenis haptic (PRD §5 mapping).
  final DashHaptic haptic;

  final double borderRadius;
  final EdgeInsetsGeometry padding;
  final Color? surfaceColor;

  @override
  ConsumerState<DashKeyButton> createState() => _DashKeyButtonState();
}

class _DashKeyButtonState extends ConsumerState<DashKeyButton>
    with TickerProviderStateMixin {
  late final AnimationController _press;
  late final AnimationController _long;
  bool _pressed = false;
  bool _longPressing = false;

  @override
  void initState() {
    super.initState();
    _press = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 120),
    );
    _long = AnimationController(
      vsync: this,
      duration: widget.longPressDuration,
    )
      ..addStatusListener((status) {
        if (status == AnimationStatus.completed && mounted) {
              // Haptic medium — sinyal siap menutup aplikasi.
          ref.read(feedbackEngineProvider.notifier).trigger(DashHaptic.medium);
          widget.onLongPress?.call();
        }
      });
  }

  @override
  void didUpdateWidget(covariant DashKeyButton oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.longPressDuration != oldWidget.longPressDuration) {
      _long.duration = widget.longPressDuration;
    }
  }

  @override
  void dispose() {
    _press.dispose();
    _long.dispose();
    super.dispose();
  }

  Duration get _pressDuration => Duration(milliseconds: (80 / speed).round());
  Duration get _releaseDuration =>
      Duration(milliseconds: (120 / speed).round());

  double get speed {
    final settings = ref.read(feedbackEngineProvider);
    return settings.animationEnabled ? settings.animationSpeed : 1.0;
  }

  Future<void> _onTapDown() async {
    if (widget.disabled || widget.onPressed == null) return;
    setState(() => _pressed = true);
    // PRD §6: haptic hampir bersamaan dengan visual press.
    await ref.read(feedbackEngineProvider.notifier).trigger(widget.haptic);
    if (mounted && _pressed) {
      _press.animateTo(
        1,
        duration: _pressDuration,
        curve: Curves.easeOut,
      );
    }
  }

  void _onTapUp() {
    if (widget.disabled) return;
    setState(() => _pressed = false);
    _press.animateTo(
      0,
      duration: _releaseDuration,
      // PRD §7: release boleh overshoot sangat kecil.
      curve: Curves.easeOutBack,
    );
    widget.onPressed?.call();
  }

  void _onTapCancel() {
    if (!_pressed) return;
    setState(() => _pressed = false);
    _press.animateTo(0, duration: _releaseDuration, curve: Curves.easeOut);
  }

  void _onLongPressStart(LongPressStartDetails _) {
    if (widget.disabled || widget.onLongPress == null) return;
    setState(() {
      _longPressing = true;
    });
    _long.duration = widget.longPressDuration;
    _long.forward(from: 0);
  }

  void _onLongPressEnd(LongPressEndDetails _) {
    _longPressing = false;
    _long.reset();
    if (mounted) setState(() {});
  }

  void _onLongPressCancel() {
    _longPressing = false;
    _long.reset();
    if (mounted) setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return IgnorePointer(
      ignoring: widget.disabled,
      child: Opacity(
        opacity: widget.disabled ? 0.45 : 1.0,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapDown: (_) => _onTapDown(),
          onTapUp: (_) => _onTapUp(),
          onTapCancel: _onTapCancel,
          // Double tap hanya aktif bila tombol punya aksi sekunder.
          onDoubleTap: widget.onDoubleTap,
          onLongPressStart: _onLongPressStart,
          onLongPressEnd: _onLongPressEnd,
          onLongPressCancel: _onLongPressCancel,
          child: AnimatedBuilder(
            animation: Listenable.merge([_press, _long]),
            builder: (context, _) {
              final t = _press.value;

              // PRD §3.1: scale 1.00 → 0.97 (jangan sampai 0.80).
              final scale = 1.0 - 0.03 * t;
              // PRD §3.2: translation 0 → 2 px.
              final dy = 2.0 * t;

              final accent = widget.accentColor ?? AppColors.blueAccent;
              final borderColor =
                  widget.active ? accent : AppColors.lightBorder;
              final borderWidth = widget.active ? 2.0 : 1.0;

              // PRD §4: shadow compress — lerp raised → pressed.
              final shadow = BoxShadow.lerp(
                AppShadows.raised.first,
                AppShadows.pressed.first,
                t,
              );

              return Transform.translate(
                offset: Offset(0, dy),
                child: Transform.scale(
                  scale: scale,
                  child: Container(
                    padding: widget.padding,
                    decoration: BoxDecoration(
                      gradient: const LinearGradient(
                        begin: Alignment.topLeft,
                        end: Alignment.bottomRight,
                        colors: [
                          AppColors.surfaceHighlight,
                          AppColors.surface,
                        ],
                      ),
                      borderRadius: BorderRadius.circular(widget.borderRadius),
                      border: Border.all(
                        color: borderColor,
                        width: borderWidth,
                      ),
                      boxShadow: [
                        shadow ?? AppShadows.raised.first,
                        if (widget.active)
                          BoxShadow(
                            color: accent.withValues(alpha: 0.25),
                            blurRadius: 18,
                            spreadRadius: 1,
                          ),
                      ],
                    ),
                    child: Stack(
                      clipBehavior: Clip.none,
                      alignment: Alignment.center,
                      children: [
                        widget.child,
                        // Ring progres long-press (indikator siap menutup).
                        if (_longPressing)
                          _LongPressRing(
                            progress: _long.value,
                            color: accent,
                          ),
                      ],
                    ),
                  ),
                ),
              );
            },
          ),
        ),
      ),
    );
  }
}

/// Ring indikator progres long press — terisi penuh = aksi akan dijalankan.
class _LongPressRing extends StatelessWidget {
  const _LongPressRing({required this.progress, required this.color});

  final double progress;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      size: const Size(46, 46),
      painter: _RingPainter(progress: progress, color: color),
    );
  }
}

class _RingPainter extends CustomPainter {
  _RingPainter({required this.progress, required this.color});

  final double progress;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final center = size.center(Offset.zero);
    final radius = size.width / 2 - 2;
    final rect = Rect.fromCircle(center: center, radius: radius);

    // Background ring.
    final bgPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 4
      ..color = AppColors.lightBorder;
    canvas.drawCircle(center, radius, bgPaint);

    // Progres.
    final fgPaint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 4
      ..strokeCap = StrokeCap.round
      ..color = color;
    canvas.drawArc(
      rect,
      -0.5 * 3.141592653589793,
      2 * 3.141592653589793 * progress.clamp(0.0, 1.0),
      false,
      fgPaint,
    );
  }

  @override
  bool shouldRepaint(covariant _RingPainter oldDelegate) =>
      oldDelegate.progress != progress || oldDelegate.color != color;
}
