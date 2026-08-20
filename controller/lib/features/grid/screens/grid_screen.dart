import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart' hide ConnectionState;
import 'package:flutter/physics.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/feedback/dashkey_haptic.dart';
import '../../../core/feedback/feedback_engine.dart';
import '../../../core/model/config.dart';
import '../../../core/protocol/messages.dart';
import '../../clock/presentation/pages/clock_page.dart';
import '../../connection/controllers/connection_controller.dart';
import '../../editor/screens/editor_home_screen.dart';
import '../../settings/settings_screen.dart';
import '../../trackpad/trackpad_screen.dart';
import '../widgets/button_icon.dart';
import '../../../shared/widgets/action_card.dart';
import '../../../shared/widgets/dashkey_button.dart';
import '../../../shared/theme/app_colors.dart';

class GridScreen extends ConsumerStatefulWidget {
  const GridScreen({super.key});

  @override
  ConsumerState<GridScreen> createState() => _GridScreenState();
}

class _GridScreenState extends ConsumerState<GridScreen> {
  PageController? _pageController;
  int _lastServerPageIndex = -1;

  @override
  void dispose() {
    _pageController?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final conn = ref.watch(connectionControllerProvider);
    final config = conn.config;
    final page = config?.currentPage;

    // Feedback results & Page Sync
    ref.listen(connectionControllerProvider, (prev, next) {
      // Snackbar feedback
      final result = next.lastActionResult;
      if (result != null && (prev?.lastActionResult != result)) {
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
      }

      // Page Sync from Server
      final activePage = next.config?.activePage;
      if (activePage != null && _pageController != null && _pageController!.hasClients) {
        final profile = next.config!.profiles.where((p) => p.profileId == next.config!.activeProfile).firstOrNull;
        final pages = profile?.pages.map((id) => next.config!.pages[id]).whereType<PageDef>().where((p) => !p.isTrackpad).toList() ?? [];
        final newIndex = pages.indexWhere((p) => p.pageId == activePage).clamp(0, pages.length - 1);
        
        if (newIndex != _lastServerPageIndex) {
          _lastServerPageIndex = newIndex;
          if (_pageController!.page?.round() != newIndex) {
            _pageController!.animateToPage(
              newIndex,
              duration: const Duration(milliseconds: 300),
              curve: Curves.easeOutCubic,
            );
          }
        }
      }
    });

    if (config == null || page == null) {
      return const Scaffold(
        body: Center(child: Text('Waiting for config from Host...')),
      );
    }

    final profile = config.profiles.where((p) => p.profileId == config.activeProfile).firstOrNull;
    final allPages = profile?.pages.map((id) => config.pages[id]).whereType<PageDef>().toList() ?? <PageDef>[];

    final pages = allPages.where((p) => !p.isTrackpad).toList();
    final trackpadPage = allPages.where((p) => p.isTrackpad).firstOrNull;

    final selectedNavIndex = pages.indexWhere((p) => p.pageId == config.activePage).clamp(0, pages.length - 1);

    if (_pageController == null) {
      _pageController = PageController(initialPage: selectedNavIndex);
      _lastServerPageIndex = selectedNavIndex;
    }

    return Scaffold(
      backgroundColor: AppColors.primaryBackground,
      body: SafeArea(
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onVerticalDragEnd: (details) {
            final velocity = details.primaryVelocity ?? 0;
            if (velocity > 300) {
              _showConfigSheet(context, ref, config, trackpadPage);
            }
          },
          child: page.isTrackpad
              ? TrackpadScreen(pageName: page.name, embedded: true)
              : PageView.builder(
                  controller: _pageController,
                  physics: const _SensitivePageScrollPhysics(),
                  onPageChanged: (index) {
                    if (index < pages.length) {
                      if (index != _lastServerPageIndex) {
                        _lastServerPageIndex = index;
                        ref.read(connectionControllerProvider.notifier).switchPage(pages[index].pageId);
                      }
                    }
                  },
                  itemCount: pages.length + 1,
                  itemBuilder: (context, pageIndex) {
                    if (pageIndex == pages.length) {
                      return const ClockPage();
                    }
                    final currentPage = pages[pageIndex];
                    return Padding(
                      padding: const EdgeInsets.all(24.0),
                      child: _buildGridForPage(context, ref, currentPage, config, conn),
                    );
                  },
                ),
        ),
      ),
    );
  }

  Widget _buildGridForPage(BuildContext context, WidgetRef ref, PageDef page, ConfigData config, ConnectionState conn) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final bool isPortrait = MediaQuery.of(context).orientation == Orientation.portrait;
        final int cols = isPortrait ? 2 : page.gridSize.cols;
        final int rows = (page.buttons.length / cols).ceil();

        const double spacing = 16.0;
        final double totalCrossAxisSpacing = (cols > 1 ? cols - 1 : 0) * spacing;
        final double totalMainAxisSpacing = (rows > 1 ? rows - 1 : 0) * spacing;

        final double itemWidth = (constraints.maxWidth - totalCrossAxisSpacing) / cols;
        final double itemHeight = (constraints.maxHeight - totalMainAxisSpacing) / rows;

        final double aspectRatio = (itemHeight > 0) ? (itemWidth / itemHeight) : 1.0;

        return GridView.builder(
          padding: EdgeInsets.zero,
          physics: const NeverScrollableScrollPhysics(),
          gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
            crossAxisCount: cols,
            mainAxisSpacing: spacing,
            crossAxisSpacing: spacing,
            childAspectRatio: aspectRatio,
          ),
          itemCount: page.buttons.length,
          itemBuilder: (context, i) {
            final buttonId = page.buttons[i];
            final button = config.buttons[buttonId];
            if (button == null) return const SizedBox.shrink();
            final dynamic = conn.buttonStates[buttonId];
            return _buildActionCard(ref, button, page.pageId, dynamic);
          },
        );
      },
    );
  }

  void _showConfigSheet(BuildContext context, WidgetRef ref, ConfigData config, PageDef? trackpadPage) {
    showModalBottomSheet(
      context: context,
      backgroundColor: AppColors.surface,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
      ),
      builder: (context) {
        return SafeArea(
          child: Padding(
            padding: const EdgeInsets.all(24.0),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 40,
                  height: 4,
                  margin: const EdgeInsets.only(bottom: 24),
                  decoration: BoxDecoration(
                    color: AppColors.lightBorder,
                    borderRadius: BorderRadius.circular(2),
                  ),
                ),
                Text('Menu & Settings', style: Theme.of(context).textTheme.titleLarge),
                const SizedBox(height: 32),
                Wrap(
                  spacing: 24,
                  runSpacing: 24,
                  alignment: WrapAlignment.center,
                  children: [
                    _buildMenuButton(context, Icons.settings_rounded, 'Settings', () {
                      Navigator.pop(context);
                      Navigator.of(context).push(MaterialPageRoute(builder: (_) => const SettingsScreen()));
                    }),
                    _buildMenuButton(context, Icons.tune_rounded, 'Editor', () {
                      Navigator.pop(context);
                      Navigator.of(context).push(MaterialPageRoute(builder: (_) => const EditorHomeScreen()));
                    }),
                    if (trackpadPage != null)
                      _buildMenuButton(context, Icons.pan_tool_rounded, 'Trackpad', () {
                        Navigator.pop(context);
                        Navigator.of(context).push(MaterialPageRoute(builder: (_) => TrackpadScreen(pageName: trackpadPage.name)));
                      }),
                    _buildMenuButton(context, Icons.link_off_rounded, 'Disconnect', () {
                      Navigator.pop(context);
                      ref.read(connectionControllerProvider.notifier).disconnect();
                    }),
                  ],
                ),
                const SizedBox(height: 16),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildMenuButton(BuildContext context, IconData icon, String label, VoidCallback onTap) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        DashKeyButton(
          onPressed: onTap,
          borderRadius: 16,
          padding: const EdgeInsets.all(16),
          child: Icon(icon, color: AppColors.primaryText, size: 28),
        ),
        const SizedBox(height: 12),
        Text(label, style: Theme.of(context).textTheme.bodySmall),
      ],
    );
  }

  Widget _buildActionCard(
    WidgetRef ref,
    ButtonDef button,
    String pageId,
    dynamic dynamicState,
  ) {
    final active = dynamicState?.state == 'active';
    final colorOverrideStr = dynamicState?.colorOverride;

    Color? overrideColor;
    if (colorOverrideStr != null) {
      overrideColor = Color(ButtonDef.parseColor(colorOverrideStr));
    }

    Widget iconWidget;
    if (active) {
      iconWidget = const Icon(
        Icons.volume_off,
        color: AppColors.primaryText,
        size: 36,
      );
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

    final fb = ref.watch(feedbackEngineProvider);

    return ActionCard(
      actionName: button.label,
      icon: iconWidget,
      accentColor: overrideColor ?? (active ? AppColors.dangerRed : null),
      active: active,
      // PRD §5: toggle/obs → selection/medium, sisanya light.
      haptic: hapticForAction(button.actions.firstOrNull?.actionType),
      longPressDuration: Duration(milliseconds: fb.longPressMs),
      onTap: () => ref
          .read(connectionControllerProvider.notifier)
          .pressButton(button.buttonId, pageId),
      // Double tap → aksi sekunder (hanya jika tombol punya).
      onDoubleTap: button.hasSecondaryActions
          ? () => ref
              .read(connectionControllerProvider.notifier)
              .pressButton(button.buttonId, pageId, ButtonGesture.doubleTap)
          : null,
      // Long press → close app (default global di Host; bisa dimatikan).
      onLongPress: fb.longPressClose
          ? () => ref
              .read(connectionControllerProvider.notifier)
              .pressButton(button.buttonId, pageId, ButtonGesture.longPress)
          : null,
    );
  }
}

class _IconImage extends StatefulWidget {
  const _IconImage({required this.iconData});
  final String iconData;

  @override
  State<_IconImage> createState() => _IconImageState();
}

class _IconImageState extends State<_IconImage> {
  Uint8List? _bytes;
  String? _lastIconData;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _decodeIfNeeded();
  }

  @override
  void didUpdateWidget(covariant _IconImage oldWidget) {
    super.didUpdateWidget(oldWidget);
    _decodeIfNeeded();
  }

  void _decodeIfNeeded() {
    if (_lastIconData != widget.iconData) {
      _lastIconData = widget.iconData;
      try {
        _bytes = base64Decode(widget.iconData);
      } catch (_) {
        _bytes = null;
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_bytes == null) {
      return const Icon(
        Icons.touch_app,
        color: AppColors.primaryText,
        size: 36,
      );
    }
    return Image.memory(
      _bytes!,
      fit: BoxFit.contain,
      errorBuilder: (_, __, ___) => const Icon(
        Icons.touch_app,
        color: AppColors.primaryText,
        size: 36,
      ),
    );
  }
}

class _SensitivePageScrollPhysics extends PageScrollPhysics {
  const _SensitivePageScrollPhysics({super.parent});

  @override
  _SensitivePageScrollPhysics applyTo(ScrollPhysics? ancestor) {
    return _SensitivePageScrollPhysics(parent: buildParent(ancestor));
  }

  @override
  Simulation? createBallisticSimulation(ScrollMetrics position, double velocity) {
    if ((velocity <= 0.0 && position.pixels <= position.minScrollExtent) ||
        (velocity >= 0.0 && position.pixels >= position.maxScrollExtent)) {
      return super.createBallisticSimulation(position, velocity);
    }
    
    final Tolerance tolerance = toleranceFor(position);
    double page = position.pixels / position.viewportDimension;
    
    if (velocity < -tolerance.velocity) {
      page -= 0.5;
    } else if (velocity > tolerance.velocity) {
      page += 0.5;
    } else {
      final double delta = page - page.roundToDouble();
      if (delta > 0.15) {
        page = page.ceilToDouble();
      } else if (delta < -0.15) {
        page = page.floorToDouble();
      } else {
        page = page.roundToDouble();
      }
    }
    
    final double target = page.roundToDouble() * position.viewportDimension;
    if (target != position.pixels) {
      return SpringSimulation(spring, position.pixels, target, velocity, tolerance: tolerance);
    }
    return null;
  }
}
