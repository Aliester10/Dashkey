/// Editor satu tombol: label, warna, dan daftar aksi berantai.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../controllers/config_editor_controller.dart';
import 'action_editor_sheet.dart';

/// Palet warna preset untuk tombol.
const buttonColorPalette = [
  '#1E88E5', '#43A047', '#E53935', '#FB8C00',
  '#8E24AA', '#3949AB', '#00ACC1', '#7B1FA2',
  '#FDD835', '#6D4C41', '#757575', '#546E7A',
];

class ButtonEditorScreen extends ConsumerWidget {
  const ButtonEditorScreen({super.key, required this.buttonId});

  final String buttonId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final draft = ref.watch(configEditorProvider).draft;
    if (draft == null) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }
    final buttons = draft['buttons'] as Map<String, dynamic>;
    final button = buttons[buttonId] as Map<String, dynamic>;
    final notifier = ref.read(configEditorProvider.notifier);

    void update(Map<String, dynamic> changes) {
      notifier.updateButton({...button, ...changes});
    }

    return Scaffold(
      appBar: AppBar(title: Text('Edit: ${button['label']}')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          TextField(
            controller: TextEditingController(text: button['label'] as String)
              ..selection = TextSelection(
                baseOffset: 0,
                extentOffset: (button['label'] as String).length,
              ),
            decoration: const InputDecoration(
              labelText: 'Label',
              border: OutlineInputBorder(),
            ),
            onChanged: (v) => update({'label': v}),
          ),
          const SizedBox(height: 20),
          Text('Warna', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (final color in buttonColorPalette)
                GestureDetector(
                  onTap: () => update({'color': color}),
                  child: Container(
                    width: 36,
                    height: 36,
                    decoration: BoxDecoration(
                      color: Color(int.parse(color.substring(1), radix: 16) |
                          0xFF000000),
                      shape: BoxShape.circle,
                      border: button['color'] == color
                          ? Border.all(color: Colors.white, width: 3)
                          : null,
                    ),
                  ),
                ),
            ],
          ),
          const SizedBox(height: 8),
          TextField(
            controller: TextEditingController(text: button['color'] as String)
              ..selection = TextSelection(
                baseOffset: 0,
                extentOffset: (button['color'] as String).length,
              ),
            decoration: const InputDecoration(
              labelText: 'Warna hex (#RRGGBB)',
              border: OutlineInputBorder(),
            ),
            onChanged: (v) => update({'color': v}),
          ),
          const SizedBox(height: 24),
          Row(
            children: [
              Text('Aksi (berurutan)',
                  style: Theme.of(context).textTheme.titleMedium),
              const Spacer(),
              TextButton.icon(
                icon: const Icon(Icons.add),
                label: const Text('Tambah Aksi'),
                onPressed: () async {
                  final action = await showModalBottomSheet<Map<String, dynamic>>(
                    context: context,
                    isScrollControlled: true,
                    builder: (_) => const ActionEditorSheet(),
                  );
                  if (action != null) {
                    final actions = [...(button['actions'] as List)];
                    actions.add(action);
                    update({'actions': actions});
                  }
                },
              ),
            ],
          ),
          const SizedBox(height: 4),
          if ((button['actions'] as List).isEmpty)
            const Padding(
              padding: EdgeInsets.all(12),
              child: Text('Belum ada aksi. Tombol tidak akan melakukan apa-apa.'),
            )
          else
            ReorderableListView.builder(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              itemCount: (button['actions'] as List).length,
              onReorder: (oldIndex, newIndex) {
                final actions = [...(button['actions'] as List)];
                if (newIndex > oldIndex) newIndex -= 1;
                final item = actions.removeAt(oldIndex);
                actions.insert(newIndex, item);
                update({'actions': actions});
              },
              itemBuilder: (context, i) {
                final action =
                    (button['actions'] as List)[i] as Map<String, dynamic>;
                return _ActionTile(
                  action: action,
                  onDelete: () {
                    final actions = [...(button['actions'] as List)];
                    actions.removeAt(i);
                    update({'actions': actions});
                  },
                  onEdit: () async {
                    final edited = await showModalBottomSheet<
                        Map<String, dynamic>>(
                      context: context,
                      isScrollControlled: true,
                      builder: (_) => ActionEditorSheet(initial: action),
                    );
                    if (edited != null) {
                      final actions = [...(button['actions'] as List)];
                      actions[i] = edited;
                      update({'actions': actions});
                    }
                  },
                );
              },
            ),
          const SizedBox(height: 28),
          const Divider(),
          const SizedBox(height: 12),
          Row(
            children: [
              Text('Aksi Sekunder (Double Tap)',
                  style: Theme.of(context).textTheme.titleMedium),
              const Spacer(),
              TextButton.icon(
                icon: const Icon(Icons.add),
                label: const Text('Tambah'),
                onPressed: () async {
                  final action = await showModalBottomSheet<Map<String, dynamic>>(
                    context: context,
                    isScrollControlled: true,
                    builder: (_) => const ActionEditorSheet(),
                  );
                  if (action != null) {
                    final secondary =
                        [...(button['secondary_actions'] as List? ?? [])];
                    secondary.add(action);
                    update({'secondary_actions': secondary});
                  }
                },
              ),
            ],
          ),
          const SizedBox(height: 4),
          Text(
            'Tap dua kali pada tombol untuk menjalankan aksi ini '
            '(contoh: Ctrl+T untuk tab baru).',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(height: 8),
          if ((button['secondary_actions'] as List? ?? []).isEmpty)
            const Padding(
              padding: EdgeInsets.all(12),
              child: Text(
                'Belum ada aksi sekunder. Tap ganda akan sama dengan tap biasa.',
              ),
            )
          else
            ...List.generate(
              (button['secondary_actions'] as List).length,
              (i) {
                final action = (button['secondary_actions'] as List)[i]
                    as Map<String, dynamic>;
                return _ActionTile(
                  action: action,
                  onDelete: () {
                    final secondary =
                        [...(button['secondary_actions'] as List)];
                    secondary.removeAt(i);
                    update({'secondary_actions': secondary});
                  },
                  onEdit: () async {
                    final edited = await showModalBottomSheet<
                        Map<String, dynamic>>(
                      context: context,
                      isScrollControlled: true,
                      builder: (_) => ActionEditorSheet(initial: action),
                    );
                    if (edited != null) {
                      final secondary =
                          [...(button['secondary_actions'] as List)];
                      secondary[i] = edited;
                      update({'secondary_actions': secondary});
                    }
                  },
                );
              },
            ),
        ],
      ),
    );
  }
}

/// Tile satu aksi di editor (label + ikon + hapus + edit).
class _ActionTile extends StatelessWidget {
  const _ActionTile({
    required this.action,
    required this.onDelete,
    required this.onEdit,
  });

  final Map<String, dynamic> action;
  final VoidCallback onDelete;
  final VoidCallback onEdit;

  @override
  Widget build(BuildContext context) {
    final type = action['action_type'] as String;
    final detail = actionLabel(type, action);
    return Card(
      child: ListTile(
        dense: true,
        leading: Icon(actionIcon(type)),
        title: Text(type),
        subtitle: detail == null ? null : Text(detail),
        trailing: IconButton(
          icon: const Icon(Icons.delete_outline),
          onPressed: onDelete,
        ),
        onTap: onEdit,
      ),
    );
  }
}
