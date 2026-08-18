import 'package:flutter/material.dart';
import '../theme/app_colors.dart';
import '../theme/app_shadows.dart';
import 'neumorphic_button.dart';

class ActionCard extends StatelessWidget {
  final String actionName;
  final Widget icon;
  final Color? accentColor;
  final VoidCallback? onTap;

  const ActionCard({
    super.key,
    required this.actionName,
    required this.icon,
    this.accentColor,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return NeumorphicButton(
      onTap: onTap,
      borderRadius: 20.0,
      padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 20.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Container(
            width: 58,
            height: 58,
            decoration: BoxDecoration(
              gradient: const LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                colors: [AppColors.surfaceHighlight, AppColors.surfaceDeep],
              ),
              borderRadius: BorderRadius.circular(18),
              border: Border.all(color: AppColors.lightBorder),
              boxShadow: AppShadows.pressed,
            ),
            child: Center(
              child: IconTheme(
                data: IconThemeData(
                  color: accentColor ?? AppColors.primaryText,
                  size: 29,
                ),
                child: icon,
              ),
            ),
          ),
          const SizedBox(height: 12),
          Text(
            actionName,
            textAlign: TextAlign.center,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.w500,
                ),
          ),
          if (accentColor != null) ...[
            const SizedBox(height: 8),
            Container(
              width: 24,
              height: 4,
              decoration: BoxDecoration(
                color: accentColor,
                borderRadius: BorderRadius.circular(2),
                boxShadow: [
                  BoxShadow(
                    color: accentColor!.withOpacity(0.5),
                    blurRadius: 8,
                    offset: const Offset(0, 2),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }
}
