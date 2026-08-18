import 'package:flutter/material.dart';
import '../theme/app_colors.dart';
import '../widgets/neumorphic_button.dart';

class ResponsiveScaffold extends StatelessWidget {
  final Widget header;
  final Widget body;
  final Widget? trailingHeader;
  final int selectedNavIndex;
  final List<Map<String, dynamic>> navItems;
  final ValueChanged<int> onNavTap;
  final VoidCallback? onSettingsTap;
  
  const ResponsiveScaffold({
    super.key,
    required this.header,
    required this.body,
    required this.navItems,
    required this.selectedNavIndex,
    required this.onNavTap,
    this.trailingHeader,
    this.onSettingsTap,
  });

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final isMobile = constraints.maxWidth < 800;

        if (isMobile) {
          return Scaffold(
            backgroundColor: AppColors.primaryBackground,
            body: SafeArea(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  // Mobile Header
                  Padding(
                    padding: const EdgeInsets.fromLTRB(24, 24, 24, 16),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Expanded(child: header),
                        if (trailingHeader != null) trailingHeader!,
                      ],
                    ),
                  ),
                  // Content
                  Expanded(
                    child: Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 24.0),
                      child: body,
                    ),
                  ),
                  // Mobile Bottom Nav
                  _buildMobileBottomNav(context),
                ],
              ),
            ),
          );
        }

        // Desktop Landscape View
        return Scaffold(
          backgroundColor: AppColors.primaryBackground,
          drawer: Theme(
            data: Theme.of(context).copyWith(
              drawerTheme: const DrawerThemeData(
                backgroundColor: Colors.transparent,
                elevation: 0,
              ),
            ),
            child: Drawer(
              width: 260,
              child: _buildDesktopSidebar(context),
            ),
          ),
          body: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Desktop Header
              Padding(
                padding: const EdgeInsets.fromLTRB(32, 32, 32, 24),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Builder(
                      builder: (context) => GestureDetector(
                        onTap: () => Scaffold.of(context).openDrawer(),
                        child: Container(
                          width: 44,
                          height: 44,
                          margin: const EdgeInsets.only(right: 24),
                          decoration: BoxDecoration(
                            color: AppColors.surface,
                            borderRadius: BorderRadius.circular(14),
                            border: Border.all(color: AppColors.lightBorder),
                          ),
                          child: const Icon(Icons.menu_rounded, color: AppColors.secondaryText, size: 24),
                        ),
                      ),
                    ),
                    Expanded(child: header),
                    if (trailingHeader != null) trailingHeader!,
                  ],
                ),
              ),
              // Content
              Expanded(
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 32.0),
                  child: body,
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildMobileBottomNav(BuildContext context) {
    if (navItems.isEmpty) return const SizedBox.shrink();
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 16.0, horizontal: 24.0),
      decoration: BoxDecoration(
        gradient: const LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [AppColors.surfaceLight, AppColors.surface],
        ),
        border: Border(
          top: BorderSide(color: AppColors.lightBorder, width: 1),
        ),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: List.generate(navItems.length, (index) {
          final isSelected = selectedNavIndex == index;
          final item = navItems[index];
          return NeumorphicButton(
              onTap: () => onNavTap(index),
              isPressedState: isSelected,
              borderRadius: 16,
              padding: const EdgeInsets.symmetric(vertical: 10, horizontal: 14),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    item['icon'] as IconData,
                    color: isSelected ? AppColors.blueAccent : AppColors.secondaryText,
                  ),
                  const SizedBox(height: 4),
                  Text(
                    item['label'] as String,
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: isSelected ? AppColors.blueAccent : AppColors.secondaryText,
                          fontWeight: isSelected ? FontWeight.w600 : FontWeight.w400,
                        ),
                  ),
                ],
              ),
            );
        }),
      ),
    );
  }

  Widget _buildDesktopSidebar(BuildContext context) {
    return Container(
      width: 260,
      decoration: const BoxDecoration(
        color: AppColors.surface,
        border: Border(
          right: BorderSide(color: AppColors.lightBorder, width: 1),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // App Logo / Title
          Padding(
            padding: const EdgeInsets.all(32.0),
            child: Row(
              children: [
                const Icon(Icons.apps_rounded, color: AppColors.blueAccent, size: 32),
                const SizedBox(width: 12),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'DASHKEY',
                      style: Theme.of(context).textTheme.titleLarge?.copyWith(
                            fontWeight: FontWeight.w700,
                            letterSpacing: 1.2,
                          ),
                    ),
                    Text(
                      'STREAM DECK',
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            letterSpacing: 1.5,
                            fontSize: 10,
                          ),
                    ),
                  ],
                ),
              ],
            ),
          ),
          // Nav Items
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.symmetric(horizontal: 16.0),
              itemCount: navItems.length,
              itemBuilder: (context, index) {
                final isSelected = selectedNavIndex == index;
                final item = navItems[index];
                return Padding(
                  padding: const EdgeInsets.only(bottom: 8.0),
                  child: NeumorphicButton(
                    onTap: () {
                      onNavTap(index);
                      if (Scaffold.of(context).isDrawerOpen) {
                        Navigator.of(context).pop();
                      }
                    },
                    isPressedState: isSelected,
                    borderRadius: 12.0,
                    padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 16.0),
                    child: Row(
                      children: [
                        Icon(
                          item['icon'] as IconData,
                          color: isSelected ? AppColors.primaryText : AppColors.secondaryText,
                          size: 20,
                        ),
                        const SizedBox(width: 16),
                        Text(
                          item['label'] as String,
                          style: Theme.of(context).textTheme.titleMedium?.copyWith(
                                color: isSelected ? AppColors.primaryText : AppColors.secondaryText,
                                fontWeight: isSelected ? FontWeight.w600 : FontWeight.w500,
                              ),
                        ),
                      ],
                    ),
                  ),
                );
              },
            ),
          ),
          // Settings
          Padding(
            padding: const EdgeInsets.all(16.0),
            child: NeumorphicButton(
              onTap: () {
                if (onSettingsTap != null) onSettingsTap!();
                if (Scaffold.of(context).isDrawerOpen) {
                  Navigator.of(context).pop();
                }
              },
              borderRadius: 12.0,
              padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 16.0),
              child: Row(
                children: [
                  const Icon(Icons.settings_outlined, color: AppColors.secondaryText, size: 20),
                  const SizedBox(width: 16),
                  Text(
                    'Settings',
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          color: AppColors.secondaryText,
                        ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
