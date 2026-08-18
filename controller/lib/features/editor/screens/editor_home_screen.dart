/// Halaman utama editor config: pilih page, urut/edit/hapus tombol,
/// tambah tombol baru, kelola pages & profiles, simpan/batal.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../controllers/config_editor_controller.dart';
import 'button_editor_screen.dart';
import 'pages_profiles_screen.dart';

class EditorHomeScreen extends ConsumerStatefulWidget {
  const EditorHomeScreen({super.key});

  @override
  ConsumerState<EditorHomeScreen> createState() => _EditorHomeScreenState();
}

class _EditorHomeScreenState extends ConsumerState<EditorHomeScreen> {
  @override
  void initState() {
    super.initState();
    Future.microtask(() {
      if (ref.read(configEditorProvider).draft == null) {
        ref.read(configEditorProvider.notifier).openEditor();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final editor = ref.watch(configEditorProvider);
    final draft = editor.draft;

    ref.listen(configEditorProvider, (prev, next) {
      if (next.lastSuccess != null && next.error == null) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(next.lastSuccess!),
            backgroundColor: Colors.green.shade700,
          ),
        );
      } else if (next.error != null) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Gagal menyimpan: ${next.error}'),
            backgroundColor: Colors.red.shade700,
          ),
        );
      } else if (next.lastSfx != null) {
        final sfx = next.lastSfx!;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(sfx.message),
            backgroundColor:
                sfx.success ? Colors.green.shade700 : Colors.red.shade700,
          ),
        );
      }
    });

    if (draft == null) {
      return Scaffold(
        appBar: AppBar(title: const Text('Editor')),
        body: const Center(child: CircularProgressIndicator()),
      );
    }

    final pages = draft['pages'] as Map<String, dynamic>;
    final buttons = draft['buttons'] as Map<String, dynamic>;
    final activeProfileId = draft['active_profile'] as String;
    final activePageId = draft['active_page'] as String;
    final profile = (draft['profiles'] as List)
        .cast<Map<String, dynamic>>()
        .firstWhere((p) => p['profile_id'] == activeProfileId);
    final profilePages = (profile['pages'] as List).cast<String>();
    final page = pages[activePageId] as Map<String, dynamic>;
    final pageButtons = (page['buttons'] as List).cast<String>();

    return Scaffold(
      appBar: AppBar(
        title: const Text('Editor Config'),
        leading: const CloseButton(),
        actions: [
          if (editor.dirty) ...[
            IconButton(
              tooltip: 'Batalkan',
              icon: const Icon(Icons.undo),
              onPressed: () => ref.read(configEditorProvider.notifier).discard(),
            ),
            IconButton(
              tooltip: 'Simpan ke Host',
              icon: editor.saving
                  ? const SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.save),
              onPressed: editor.saving
                  ? null
                  : () => ref.read(configEditorProvider.notifier).save(),
            ),
          ],
        ],
      ),
      body: Column(
        children: [
          // Pilih page dari profile aktif.
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 12, 12, 4),
            child: Align(
              alignment: Alignment.centerLeft,
              child: SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: Row(
                  children: [
                    for (final pageId in profilePages)
                      Padding(
                        padding: const EdgeInsets.only(right: 8),
                        child: ChoiceChip(
                          label: Text(
                            (pages[pageId] as Map<String, dynamic>)['name']
                                as String,
                          ),
                          selected: pageId == activePageId,
                          onSelected: (_) => ref
                              .read(configEditorProvider.notifier)
                              .setActivePage(pageId),
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
          const Divider(height: 8),
          // Tombol pada page aktif.
          Expanded(
            child: pageButtons.isEmpty
                ? const Center(child: Text('Belum ada tombol di page ini'))
                : ReorderableListView.builder(
                    padding: const EdgeInsets.symmetric(horizontal: 12),
                    itemCount: pageButtons.length,
                    onReorder: (oldIndex, newIndex) => ref
                        .read(configEditorProvider.notifier)
                        .moveButton(activePageId, oldIndex, newIndex),
                    itemBuilder: (context, i) {
                      final buttonId = pageButtons[i];
                      final button =
                          buttons[buttonId] as Map<String, dynamic>;
                      return Card(
                        key: ValueKey(buttonId),
                        child: ListTile(
                          leading: const Icon(Icons.drag_indicator),
                          tileColor: Color(int.parse(
                                (button['color'] as String).substring(1),
                                radix: 16,
                              ) |
                              0xFF000000),
                          title: Text(
                            button['label'] as String,
                            style: const TextStyle(color: Colors.white),
                          ),
                          subtitle: Text(
                            ((button['actions'] as List?) ?? [])
                                .length
                                .toString() +
                                ' aksi',
                            style: const TextStyle(color: Colors.white70),
                          ),
                          onTap: () async {
                            await Navigator.of(context).push(
                              MaterialPageRoute(
                                builder: (_) =>
                                    ButtonEditorScreen(buttonId: buttonId),
                              ),
                            );
                            setState(() {});
                          },
                          trailing: IconButton(
                            icon: const Icon(Icons.delete_outline,
                                color: Colors.white70),
                            onPressed: () => ref
                                .read(configEditorProvider.notifier)
                                .deleteButton(buttonId),
                          ),
                        ),
                      );
                    },
                  ),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () {
          ref.read(configEditorProvider.notifier).addButton(activePageId);
        },
        icon: const Icon(Icons.add),
        label: const Text('Tombol'),
      ),
      bottomNavigationBar: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(8),
          child: Row(
            children: [
              Expanded(
                child: OutlinedButton.icon(
                  icon: const Icon(Icons.music_note_outlined),
                  label: const Text('Import SFX'),
                  onPressed: () => _showSfxImportDialog(context),
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: OutlinedButton.icon(
                  icon: const Icon(Icons.web_stories_outlined),
                  label: const Text('Pages & Profiles'),
                  onPressed: () => Navigator.of(context).push(
                    MaterialPageRoute(
                      builder: (_) => const PagesProfilesScreen(),
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  /// Dialog paste kode embed/URL MyInstants → kirim import_sfx.
  Future<void> _showSfxImportDialog(BuildContext context) async {
    final controller = TextEditingController();
    final result = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Import SFX dari MyInstants'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text(
              'Buka www.myinstants.com → pilih suara → copy kode '
              '"Embed this button to your site!" lalu paste di sini. '
              'Bisa juga paste URL halaman/embed/mp3-nya.',
              style: TextStyle(fontSize: 13),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: controller,
              maxLines: 4,
              decoration: const InputDecoration(
                hintText: '<iframe ...myinstants.com/instant/.../embed/...',
                border: OutlineInputBorder(),
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Batal'),
          ),
          FilledButton.icon(
            icon: const Icon(Icons.download),
            label: const Text('Import'),
            onPressed: () => Navigator.pop(context, controller.text.trim()),
          ),
        ],
      ),
    );
    if (result == null || result.isEmpty) return;
    ref.read(configEditorProvider.notifier).importSfx(result);
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Mengimpor SFX... (unduh di PC)')),
    );
  }
}
