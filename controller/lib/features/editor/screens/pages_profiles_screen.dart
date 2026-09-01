/// Kelola pages & profiles (FR-7..10): buat/rename/hapus page,
/// set ukuran grid, buat/rename/hapus profile.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../controllers/config_editor_controller.dart';

class PagesProfilesScreen extends ConsumerWidget {
  const PagesProfilesScreen({super.key});

  Future<void> _prompt(
    BuildContext context, {
    required String title,
    String initial = '',
    required void Function(String value) onSave,
  }) async {
    final controller = TextEditingController(text: initial);
    final result = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: TextField(controller: controller, autofocus: true),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Batal'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, controller.text.trim()),
            child: const Text('Simpan'),
          ),
        ],
      ),
    );
    if (result != null && result.isNotEmpty) onSave(result);
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final draft = ref.watch(configEditorProvider).draft;
    if (draft == null) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }
    final notifier = ref.read(configEditorProvider.notifier);
    final pages = draft['pages'] as Map<String, dynamic>;
    final profiles = (draft['profiles'] as List).cast<Map<String, dynamic>>();

    return Scaffold(
      appBar: AppBar(title: const Text('Pages & Profiles')),
      body: ListView(
        padding: const EdgeInsets.all(12),
        children: [
          Row(
            children: [
              Text('Pages', style: Theme.of(context).textTheme.titleLarge),
              const Spacer(),
              TextButton.icon(
                icon: const Icon(Icons.add),
                label: const Text('Page Baru'),
                onPressed: () => notifier.addPage(),
              ),
            ],
          ),
          for (final entry in pages.entries.toList())
            Card(
              child: Column(
                children: [
                  ListTile(
                    title: Text(
                      (entry.value as Map<String, dynamic>)['name'] as String,
                    ),
                    subtitle: Text(
                      'Grid ${(entry.value['grid_size'] as Map)['rows']}x${(entry.value['grid_size'] as Map)['cols']} • '
                      '${(entry.value['buttons'] as List).whereType<String>().length} tombol',
                    ),
                    trailing: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        IconButton(
                          icon: const Icon(Icons.edit),
                          onPressed: () => _prompt(
                            context,
                            title: 'Rename Page',
                            initial: entry.value['name'] as String,
                            onSave: (v) => notifier.updatePage({
                              ...entry.value as Map<String, dynamic>,
                              'name': v,
                            }),
                          ),
                        ),
                        IconButton(
                          icon: const Icon(Icons.grid_on),
                          onPressed: () => _editGridSize(
                            context,
                            ref,
                            entry.key,
                            entry.value as Map<String, dynamic>,
                          ),
                        ),
                        IconButton(
                          icon: const Icon(Icons.delete_outline),
                          onPressed: pages.length > 1
                              ? () => notifier.deletePage(entry.key)
                              : null,
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          const Divider(height: 24),
          Row(
            children: [
              Text('Profiles', style: Theme.of(context).textTheme.titleLarge),
              const Spacer(),
              TextButton.icon(
                icon: const Icon(Icons.add),
                label: const Text('Profile Baru'),
                onPressed: () => notifier.addProfile(),
              ),
            ],
          ),
          for (final profile in profiles)
            Card(
              child: ListTile(
                title: Text(profile['name'] as String),
                subtitle: Text('${(profile['pages'] as List).length} page'),
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    IconButton(
                      icon: const Icon(Icons.edit),
                      onPressed: () => _prompt(
                        context,
                        title: 'Rename Profile',
                        initial: profile['name'] as String,
                        onSave: (v) => notifier.updateProfile({
                          ...profile,
                          'name': v,
                        }),
                      ),
                    ),
                    IconButton(
                      icon: const Icon(Icons.delete_outline),
                      onPressed: profiles.length > 1
                          ? () => notifier
                              .deleteProfile(profile['profile_id'] as String)
                          : null,
                    ),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }

  /// Dialog set ukuran grid page.
  Future<void> _editGridSize(
    BuildContext context,
    WidgetRef ref,
    String pageId,
    Map<String, dynamic> page,
  ) async {
    var rows = (page['grid_size'] as Map)['rows'] as int;
    var cols = (page['grid_size'] as Map)['cols'] as int;
    final result = await showDialog<({int rows, int cols})>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: Text('Grid ${page['name']}'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  IconButton(
                    onPressed: rows > 1 ? () => setState(() => rows--) : null,
                    icon: const Icon(Icons.remove_circle_outline),
                  ),
                  Text('Baris: $rows', style: const TextStyle(fontSize: 18)),
                  IconButton(
                    onPressed: rows < 8 ? () => setState(() => rows++) : null,
                    icon: const Icon(Icons.add_circle_outline),
                  ),
                ],
              ),
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  IconButton(
                    onPressed: cols > 1 ? () => setState(() => cols--) : null,
                    icon: const Icon(Icons.remove_circle_outline),
                  ),
                  Text('Kolom: $cols', style: const TextStyle(fontSize: 18)),
                  IconButton(
                    onPressed: cols < 8 ? () => setState(() => cols++) : null,
                    icon: const Icon(Icons.add_circle_outline),
                  ),
                ],
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Batal'),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, (rows: rows, cols: cols)),
              child: const Text('Simpan'),
            ),
          ],
        ),
      ),
    );
    if (result == null) return;
    final notifier = ref.read(configEditorProvider.notifier);
    notifier.updatePage({
      ...page,
      'grid_size': {'rows': result.rows, 'cols': result.cols},
    });
  }
}
