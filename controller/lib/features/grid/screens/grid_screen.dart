import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/model/config.dart';
import '../../connection/controllers/connection_controller.dart';
import '../../editor/screens/editor_home_screen.dart';
import '../../settings/settings_screen.dart';
import '../../trackpad/trackpad_screen.dart';
import '../widgets/button_icon.dart';
import '../../../shared/layout/responsive_scaffold.dart';
import '../../../shared/widgets/action_card.dart';
import '../../../shared/widgets/neumorphic_button.dart';
import '../../../shared/theme/app_colors.dart';

class GridScreen extends ConsumerWidget {
  const GridScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final conn = ref.watch(connectionControllerProvider);
    final config = conn.config;
    final page = config?.currentPage;
    
    // Feedback results
    ref.listen(connectionControllerProvider, (prev, next) {
      final result = next.lastActionResult;
      if (result == null) return;
      final label = next.config?.buttons[result.buttonId]?.label ?? result.buttonId;
      ScaffoldMessenger.of(context)
        ..hideCurrentSnackBar()
        ..showSnackBar(
          SnackBar(
            content: Text(
              result.success
                  ? '[$label] OK${result.message != null ? " — ${result.message}" : ""}'
                  : '[$label] GAGAL: ${result.message ?? "error tidak dikenal"}',
            ),
            backgroundColor: result.success ? AppColors.successGreen : AppColors.dangerRed,
            duration: const Duration(seconds: 2),
            behavior: SnackBarBehavior.floating,
          ),
        );
    });

    if (config == null || page == null) {
      return const Scaffold(
        body: Center(child: Text('Waiting for config from Host...')),
      );
    }

    final profile = config.profiles
        .where((p) => p.profileId == config.activeProfile)
        .firstOrNull;
    final allPages = profile?.pages
            .map((id) => config.pages[id])
            .whereType<PageDef>()
            .toList() ??
        <PageDef>[];

    final pages = allPages.where((p) => !p.isTrackpad).toList();
    final trackpadPage = allPages.where((p) => p.isTrackpad).firstOrNull;
    
    final selectedNavIndex = pages.indexWhere((p) => p.pageId == config.activePage).clamp(0, pages.length - 1);
    
    final navItems = pages.map((p) => {
      'label': p.name,
      'icon': p.name.toLowerCase().contains('media') ? Icons.play_arrow_rounded 
          : p.name.toLowerCase().contains('obs') ? Icons.desktop_windows_rounded 
          : Icons.grid_view_rounded,
    }).toList();

    return ResponsiveScaffold(
      navItems: navItems,
      selectedNavIndex: selectedNavIndex,
      onNavTap: (i) {
        if (i >= 0 && i < pages.length) {
          ref.read(connectionControllerProvider.notifier).switchPage(pages[i].pageId);
        }
      },
      onSettingsTap: () => Navigator.of(context).push(
        MaterialPageRoute(builder: (_) => const SettingsScreen()),
      ),
      header: _buildHeader(context, page.name, page.buttons.length),
      trailingHeader: _buildHeaderActions(context, ref, config, trackpadPage),
      body: page.isTrackpad
          ? TrackpadScreen(pageName: page.name, embedded: true)
          : GridView.builder(
              padding: const EdgeInsets.only(bottom: 24),
              gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                crossAxisCount: MediaQuery.of(context).size.width < 800 ? 2 : page.gridSize.cols,
                mainAxisSpacing: 16,
                crossAxisSpacing: 16,
                childAspectRatio: 1.1,
              ),
              itemCount: page.buttons.length,
              itemBuilder: (context, i) {
                final buttonId = page.buttons[i];
                final button = config.buttons[buttonId];
                if (button == null) return const SizedBox.shrink();
                final dynamic = conn.buttonStates[buttonId];
                return _buildActionCard(ref, button, page.pageId, dynamic);
              },
            ),
    );
  }

  Widget _buildHeader(BuildContext context, String title, int actionCount) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(title, style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: 4),
        Text('$actionCount Actions', style: Theme.of(context).textTheme.bodySmall),
      ],
    );
  }

  Widget _buildHeaderActions(BuildContext context, WidgetRef ref, ConfigData config, PageDef? trackpadPage) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (trackpadPage != null)
          _buildIconButton(
            context,
            Icons.pan_tool_rounded,
            onTap: () => Navigator.of(context).push(
              MaterialPageRoute(builder: (_) => TrackpadScreen(pageName: trackpadPage.name)),
            ),
          ),
        const SizedBox(width: 12),
        _buildIconButton(
          context,
          Icons.tune_rounded,
          onTap: () => Navigator.of(context).push(
            MaterialPageRoute(builder: (_) => const EditorHomeScreen()),
          ),
        ),
        const SizedBox(width: 12),
        if (config.profiles.length > 1)
          _buildIconButton(
            context,
            Icons.account_tree_outlined,
            onTap: () {
              // TODO: implement profile switch menu with neumorphic style
            },
          ),
        const SizedBox(width: 12),
        _buildIconButton(
          context,
          Icons.link_off_rounded,
          onTap: () => ref.read(connectionControllerProvider.notifier).disconnect(),
        ),
      ],
    );
  }

  Widget _buildIconButton(BuildContext context, IconData icon, {VoidCallback? onTap}) {
    return NeumorphicButton(
      onTap: onTap,
      borderRadius: 14,
      padding: const EdgeInsets.all(11),
      child: Icon(icon, color: AppColors.secondaryText, size: 20),
    );
  }

  Widget _buildActionCard(WidgetRef ref, ButtonDef button, String pageId, dynamic dynamicState) {
    final active = dynamicState?.state == 'active';
    final colorOverrideStr = dynamicState?.colorOverride;
    
    Color? overrideColor;
    if (colorOverrideStr != null) {
      overrideColor = Color(ButtonDef.parseColor(colorOverrideStr));
    }
    
    Widget iconWidget;
    if (active) {
      iconWidget = const Icon(Icons.volume_off, color: AppColors.primaryText, size: 36);
    } else if (button.iconData != null) {
      iconWidget = _IconImage(iconData: button.iconData!);
    } else if (iconForKey(button.icon, button.actions) case final icon?) {
      iconWidget = Icon(icon, color: AppColors.primaryText, size: 36);
    } else {
      iconWidget = Text(
        button.label.isEmpty ? '?' : button.label[0].toUpperCase(),
        style: const TextStyle(
          color: AppColors.primaryText,
          fontSize: 24,
          fontWeight: FontWeight.bold,
        ),
      );
    }

    return ActionCard(
      actionName: button.label,
      icon: iconWidget,
      accentColor: overrideColor ?? (active ? AppColors.dangerRed : null),
      onTap: () => ref.read(connectionControllerProvider.notifier).pressButton(button.buttonId, pageId),
    );
  }
}

class _IconImage extends StatelessWidget {
  const _IconImage({required this.iconData});
  final String iconData;
  @override
  Widget build(BuildContext context) {
    try {
      final bytes = base64Decode(iconData);
      return Image.memory(
        bytes,
        fit: BoxFit.contain,
        errorBuilder: (_, __, ___) => const Icon(Icons.touch_app, color: AppColors.primaryText, size: 36),
      );
    } catch (_) {
      return const Icon(Icons.touch_app, color: AppColors.primaryText, size: 36);
    }
  }
}
