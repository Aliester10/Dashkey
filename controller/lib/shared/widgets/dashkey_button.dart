/// DashKeyButton — tombol reusable dengan Digital Tactile Feedback (prdbutton.md).
///
/// Fitur:
/// - State: normal / pressed / active / disabled (PRD §2).
/// - Press: scale 1.00→0.97 + translate 0→2px + shadow compress (PRD §3–§4).
/// - Timing: press 80ms (easeOut), release 120ms (easeOutBack, overshoot kecil)
///   — PRD §6–§7. Semua disesuaikan `animationSpeed` dari Settings.
/// - Haptic & sound opsional lewat FeedbackEngine (PRD §5, §8, §13).
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
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;
  bool _pressed = false;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 120),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Duration get _pressDuration =>
      Duration(milliseconds: (80 / speed).round());
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
      _controller.animateTo(
        1,
        duration: _pressDuration,
        curve: Curves.easeOut,
      );
    }
  }

  void _onTapUp() {
    if (widget.disabled) return;
    setState(() => _pressed = false);
    _controller.animateTo(
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
    _controller.animateTo(0, duration: _releaseDuration, curve: Curves.easeOut);
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
          child: AnimatedBuilder(
            animation: _controller,
            builder: (context, _) {
              final t = _controller.value;

              // PRD §3.1: scale 1.00 → 0.97 (jangan sampai 0.80).
              final scale = 1.0 - 0.03 * t;
              // PRD §3.2: translation 0 → 2 px.
              final dy = 2.0 * t;

              final accent = widget.accentColor ?? AppColors.blueAccent;
              final borderColor = widget.active
                  ? accent
                  : AppColors.lightBorder;
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
                    child: widget.child,
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
