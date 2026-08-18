import 'package:flutter/material.dart';
import '../theme/app_colors.dart';
import '../theme/app_shadows.dart';

class NeumorphicButton extends StatefulWidget {
  final Widget child;
  final VoidCallback? onTap;
  final double borderRadius;
  final EdgeInsetsGeometry padding;
  final Color? surfaceColor;
  final bool isPressedState;
  final double scaleOnPress;

  const NeumorphicButton({
    super.key,
    required this.child,
    this.onTap,
    this.borderRadius = 16.0,
    this.padding = const EdgeInsets.all(16.0),
    this.surfaceColor,
    this.isPressedState = false,
    this.scaleOnPress = 0.97,
  });

  @override
  State<NeumorphicButton> createState() => _NeumorphicButtonState();
}

class _NeumorphicButtonState extends State<NeumorphicButton> {
  bool _isHovering = false;
  bool _isPressing = false;

  bool get _isActive => _isPressing || widget.isPressedState;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) {
        if (widget.onTap != null) setState(() => _isHovering = true);
      },
      onExit: (_) {
        if (widget.onTap != null) setState(() => _isHovering = false);
      },
      child: GestureDetector(
        onTapDown: (_) {
          if (widget.onTap != null) setState(() => _isPressing = true);
        },
        onTapUp: (_) {
          if (widget.onTap != null) setState(() => _isPressing = false);
          widget.onTap?.call();
        },
        onTapCancel: () {
          if (widget.onTap != null) setState(() => _isPressing = false);
        },
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 150),
          curve: Curves.easeOut,
          transform: Matrix4.identity()..scale(_isActive ? widget.scaleOnPress : 1.0),
          transformAlignment: Alignment.center,
          padding: widget.padding,
          decoration: BoxDecoration(
            color: widget.surfaceColor ?? AppColors.surface,
            gradient: LinearGradient(
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
              colors: _isActive
                  ? [AppColors.surfaceDeep, AppColors.surface]
                  : [AppColors.surfaceHighlight, AppColors.surface],
            ),
            borderRadius: BorderRadius.circular(widget.borderRadius),
            border: Border.all(
              color: AppColors.lightBorder,
              width: 1,
            ),
            boxShadow: _isActive
                ? AppShadows.pressed
                : _isHovering
                    ? AppShadows.hover
                    : AppShadows.raised,
          ),
          child: widget.child,
        ),
      ),
    );
  }
}
