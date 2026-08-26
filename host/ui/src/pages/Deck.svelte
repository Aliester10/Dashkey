<script lang="ts">
  import { onDestroy, onMount, untrack } from "svelte";
  import { ICON } from "../lib/icons";
  import {
    addButtonAt,
    addPlaySound,
    createButton,
    createPage,
    deleteButton,
    fileIconSrc,
    getSnapshot,
    listSounds,
    moveButton,
    openSoundsFolder,
    playSound,
    runAction,
    scanApps,
    setActivePage,
    setActiveProfile,
    setButtonIconFile,
    testButton,
    updateButton,
  } from "../lib/api";
  import { BUTTON_COLORS, describeAction, ICON_OPTIONS } from "../lib/constants";
  import { getConfirmCtx } from "../lib/confirm.svelte";
  import { getThemeCtx } from "../lib/theme.svelte";
  import type { Action, Button, Config, DetectedApp, Page, StatusPayload } from "../lib/types";
  import { open } from "@tauri-apps/plugin-dialog";
  import ActionEditorModal from "../components/ActionEditorModal.svelte";
  import SidebarSection from "../components/SidebarSection.svelte";

  const confirm = getConfirmCtx();
  const themeCtx = getThemeCtx();

  let {
    config,
    status,
    onMutate,
    onOpenManage,
  }: {
    config: Config;
    status: StatusPayload;
    onMutate: () => Promise<void>;
    onOpenManage: () => void;
  } = $props();

  // ── State ────────────────────────────────────────────────────────────
  let selectedPage = $state(untrack(() => config.active_page));
  let selectedButtonId = $state<string | null>(null);
  let labelDraft = $state("");
  let showEditor = $state(false);
  let testResult = $state("");
  let dropTarget = $state<number | null>(null);

  // Accordion sidebar
  let appsOpen = $state(false);
  let soundOpen = $state(false);
  let systemOpen = $state(false);

  // Drag payload (Elgato style: apps / sound / quick action / tombol)
  let dragPayload = $state<DragPayload | null>(null);
  let dragPos = $state<{ x: number; y: number } | null>(null);
  let tileDrag = $state<{ buttonId: string; fromIdx: number; startX: number; startY: number } | null>(null);
  let suppressClick = $state(false);

  let apps = $state<DetectedApp[]>([]);
  let appSearch = $state("");
  let appsBusy = $state(false);
  let sounds = $state<string[]>([]);
  let soundBusy = $state(false);

  type DragPayload =
    | { kind: "app"; app: DetectedApp; label: string }
    | { kind: "sound"; sound: string; label: string }
    | { kind: "action"; action: Record<string, unknown>; label: string }
    | { kind: "tile"; buttonId: string; fromIdx: number; label: string };

  // ── Derived ──────────────────────────────────────────────────────────
  const activeProfile = $derived(
    config.profiles.find((p) => p.profile_id === config.active_profile),
  );
  const profilePages = $derived(
    (activeProfile?.pages ?? [])
      .map((id) => config.pages[id])
      .filter((p): p is Page => !!p)
      .sort((a, b) => a.page_id.localeCompare(b.page_id)),
  );
  const page = $derived(config.pages[selectedPage]);
  const pageButtons = $derived(page?.buttons ?? []);
  const rows = $derived(page?.grid_size.rows ?? 4);
  const cols = $derived(page?.grid_size.cols ?? 4);
  const pageIdx = $derived(profilePages.findIndex((p) => p.page_id === selectedPage));
  const selectedButton = $derived(selectedButtonId ? config.buttons[selectedButtonId] : undefined);
  const filteredApps = $derived(
    apps.filter((a) => !appSearch || a.name.toLowerCase().includes(appSearch.toLowerCase())),
  );
  const online = $derived(status.connectionCount > 0);

  const mediaActions: { label: string; glyph: string; control: string }[] = [
    { label: "Play/Pause", glyph: "⏯", control: "play_pause" },
    { label: "Next", glyph: "⏭", control: "next" },
    { label: "Prev", glyph: "⏮", control: "prev" },
    { label: "Stop", glyph: "⏹", control: "stop" },
    { label: "Vol +", glyph: "🔊", control: "volume_up" },
    { label: "Vol −", glyph: "🔉", control: "volume_down" },
    { label: "Mute", glyph: "🔇", control: "mute" },
  ];

  const systemLinks: { label: string; url: string }[] = [
    { label: "GitHub — DashKey", url: "https://github.com/Aliester10/Dashkey" },
  ];

  $effect(() => {
    if (selectedButton) labelDraft = selectedButton.label;
  });

  // Pastikan page terpilih selalu ada di profile aktif.
  $effect(() => {
    if (profilePages.length > 0 && !profilePages.some((p) => p.page_id === selectedPage)) {
      selectedPage = profilePages[0].page_id;
    }
  });

  function semanticGlyph(icon: string | null | undefined): string {
    if (!icon || icon.startsWith("file://")) return "⚡";
    const opt = ICON_OPTIONS.find((o) => o.key === icon);
    if (!opt?.label) return "⚡";
    return opt.label.split(" ")[0] ?? "⚡";
  }

  // ── Data sidebar ─────────────────────────────────────────────────────
  async function refreshApps() {
    appsBusy = true;
    try {
      apps = await scanApps();
    } finally {
      appsBusy = false;
    }
  }

  async function refreshSounds() {
    soundBusy = true;
    try {
      sounds = await listSounds();
    } finally {
      soundBusy = false;
    }
  }

  onMount(() => {
    refreshApps();
    refreshSounds();
    // Pointer-based drag & drop (HTML5 DnD tidak stabil di WebView2).
    window.addEventListener("pointermove", windowPointerMove);
    window.addEventListener("pointerup", windowPointerUp);
    window.addEventListener("pointercancel", endDrag);
  });

  onDestroy(() => {
    window.removeEventListener("pointermove", windowPointerMove);
    window.removeEventListener("pointerup", windowPointerUp);
    window.removeEventListener("pointercancel", endDrag);
  });

  // ── Mutasi ───────────────────────────────────────────────────────────
  async function mutate(fn: () => Promise<unknown>) {
    await fn();
    await onMutate();
  }

  async function selectPage(id: string) {
    selectedPage = id;
    selectedButtonId = null;
    await mutate(() => setActivePage(id));
  }

  async function addKeyAt(label = "") {
    const btn = await createButton(selectedPage, label);
    selectedButtonId = btn.button_id;
    await onMutate();
  }

  function clickTile(buttonId: string | undefined) {
    if (suppressClick) {
      suppressClick = false;
      return;
    }
    if (buttonId && config.buttons[buttonId]) {
      selectedButtonId = selectedButtonId === buttonId ? null : buttonId;
    } else {
      addKeyAt("");
    }
  }

  // ── Navigation ───────────────────────────────────────────────────────
  async function switchProfile(profileId: string) {
    if (profileId === config.active_profile) return;
    await mutate(() => setActiveProfile(profileId));
  }

  async function goPrevPage() {
    if (pageIdx > 0) await selectPage(profilePages[pageIdx - 1].page_id);
  }

  async function goNextPage() {
    if (pageIdx >= 0 && pageIdx < profilePages.length - 1) {
      await selectPage(profilePages[pageIdx + 1].page_id);
    }
  }

  async function createFolder() {
    if (!activeProfile) return;
    await mutate(() => createPage(activeProfile.profile_id));
    const updated = await getSnapshot();
    const prof = updated.profiles.find((p) => p.profile_id === updated.active_profile);
    const lastPage = prof?.pages[prof.pages.length - 1];
    if (lastPage) await selectPage(lastPage);
  }

  // ── Drag & drop (pointer events) ─────────────────────────────────────
  function rowPointerDown(e: PointerEvent, payload: DragPayload) {
    if (e.button !== 0) return;
    e.preventDefault();
    dragPayload = payload;
    dragPos = { x: e.clientX, y: e.clientY };
    dropTarget = null;
  }

  /** Mulai drag tombol dari tile grid (belum tentu drag — bisa klik). */
  function tilePointerDown(e: PointerEvent, buttonId: string, slotIdx: number) {
    if (e.button !== 0) return;
    tileDrag = { buttonId, fromIdx: slotIdx, startX: e.clientX, startY: e.clientY };
    suppressClick = false;
  }

  /** Cari indeks slot grid di bawah koordinat kursor. */
  function slotAt(x: number, y: number): number | null {
    const el = document.elementFromPoint(x, y);
    const tile = el?.closest("[data-slot]") as HTMLElement | null;
    if (!tile) return null;
    const idx = Number(tile.dataset.slot);
    return Number.isInteger(idx) ? idx : null;
  }

  function windowPointerMove(e: PointerEvent) {
    if (!dragPayload) {
      // Promosi klik-tahan tile menjadi drag setelah gerakan minimal.
      if (tileDrag && Math.hypot(e.clientX - tileDrag.startX, e.clientY - tileDrag.startY) > 5) {
        const btn = config.buttons[tileDrag.buttonId];
        dragPayload = {
          kind: "tile",
          buttonId: tileDrag.buttonId,
          fromIdx: tileDrag.fromIdx,
          label: btn?.label || "…",
        };
        dragPos = { x: e.clientX, y: e.clientY };
        dropTarget = null;
      }
      return;
    }
    e.preventDefault();
    dragPos = { x: e.clientX, y: e.clientY };
    dropTarget = slotAt(e.clientX, e.clientY);
  }

  function endDrag() {
    dragPayload = null;
    dragPos = null;
    dropTarget = null;
    tileDrag = null;
  }

  function windowPointerUp(e: PointerEvent) {
    if (!dragPayload) {
      tileDrag = null;
      return;
    }
    const payload = dragPayload;
    const idx = dropTarget;
    endDrag();
    if (payload.kind === "tile") {
      suppressClick = true;
      if (idx !== null) performTileDrop(payload, idx);
    } else if (idx !== null) {
      performDrop(payload, idx);
    }
  }

  /** Pindahkan tombol antar slot (drag & drop tombol). */
  async function performTileDrop(payload: Extract<DragPayload, { kind: "tile" }>, toIdx: number) {
    if (toIdx === payload.fromIdx) return;
    await mutate(() => moveButton(selectedPage, payload.fromIdx, toIdx));
    if (selectedButtonId === payload.buttonId || selectedButtonId === pageButtons[toIdx]) {
      selectedButtonId = payload.buttonId;
    }
  }

  /** Bangun tombol dari payload drag (app / sound / quick action). */
  function buildButton(payload: DragPayload): Button {
    const slug = payload.label.toLowerCase().replace(/[^a-z0-9]+/g, "_");
    switch (payload.kind) {
      case "app": {
        const app = payload.app;
        return {
          button_id: `btn_app_${slug}_${Date.now()}`,
          label: app.name,
          icon: app.icon_path ? `file://${app.icon_path}` : null,
          color: "#00ACC1",
          actions: [{ action_type: "open_app", target: app.target }],
          secondary_actions: [],
        };
      }
      case "sound":
        return {
          button_id: `btn_sound_${slug}_${Date.now()}`,
          label: payload.label,
          icon: "music",
          color: "#EFA94B",
          actions: [{ action_type: "play_sound", target: payload.sound }],
          secondary_actions: [],
        };
      case "action":
        return {
          button_id: `btn_action_${slug}_${Date.now()}`,
          label: payload.label,
          icon: null,
          color: "#8B5CF6",
          actions: [{ ...payload.action } as Action],
          secondary_actions: [],
        };
      case "tile":
        // Tidak pernah dipakai: drag tombol memakai performTileDrop.
        return {
          button_id: `btn_tile_${Date.now()}`,
          label: payload.label,
          icon: null,
          color: "#00ACC1",
          actions: [],
          secondary_actions: [],
        };
    }
  }

  async function performDrop(payload: DragPayload, slotIdx: number) {
    const existingId = pageButtons[slotIdx];
    const existing = existingId ? config.buttons[existingId] : undefined;
    const button = buildButton(payload);

    if (existing && existing.actions.length > 0) {
      const ok = await confirm.requestConfirm({
        title: "Ganti tombol?",
        message: `Slot ini sudah berisi "${existing.label}". Ganti dengan "${button.label}"?`,
        confirmLabel: "Ganti",
      });
      if (!ok) return;
      await mutate(() => updateButton({ ...existing, ...button, button_id: existing.button_id }));
    } else if (existing) {
      await mutate(() => updateButton({ ...existing, ...button, button_id: existing.button_id }));
    } else {
      await mutate(() => addButtonAt(selectedPage, button, slotIdx));
    }
    selectedButtonId = existingId ?? null;
  }

  // ── Quick actions (System) ───────────────────────────────────────────
  async function doQuickAction(action: Record<string, unknown>) {
    try {
      await runAction(action);
    } catch (e) {
      testResult = String(e);
    }
  }

  // ── Config tombol ────────────────────────────────────────────────────
  async function commitLabel() {
    if (!selectedButton) return;
    const label = labelDraft.trim();
    if (!label || label === selectedButton.label) return;
    await mutate(() => updateButton({ ...selectedButton, label }));
  }

  async function setColor(color: string) {
    if (!selectedButton) return;
    await mutate(() => updateButton({ ...selectedButton, color }));
  }

  async function setIcon(key: string | null) {
    if (!selectedButton) return;
    await mutate(() => updateButton({ ...selectedButton, icon: key }));
  }

  async function pickIconFile() {
    if (!selectedButton) return;
    const path = await open({
      multiple: false,
      filters: [{ name: "Gambar", extensions: ["png", "jpg", "jpeg", "svg", "ico"] }],
    });
    if (typeof path === "string") {
      await mutate(() => setButtonIconFile(selectedButton!.button_id, path));
    }
  }

  async function pickSound() {
    if (!selectedButton) return;
    const path = await open({
      multiple: false,
      filters: [{ name: "Audio", extensions: ["mp3", "wav", "ogg", "m4a", "flac"] }],
    });
    if (typeof path === "string") {
      await mutate(() => addPlaySound(selectedButton!.button_id, path));
    }
  }

  async function doDelete() {
    if (!selectedButton) return;
    const ok = await confirm.requestConfirm({
      title: "Delete button?",
      message: `Button "${selectedButton.label}" dan seluruh aksinya akan dihapus.`,
      confirmLabel: "Hapus",
      danger: true,
    });
    if (!ok) return;
    const id = selectedButtonId!;
    selectedButtonId = null;
    await mutate(() => deleteButton(id));
  }

  async function doTest() {
    if (!selectedButton) return;
    testResult = await testButton(selectedButton.button_id);
  }
</script>

<div class="flex h-full">
  <!-- Sidebar ala Elgato -->
  <aside class="flex h-full w-[300px] shrink-0 flex-col border-r border-border bg-surface-1/60">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 pb-2 pt-4">
      <div class="flex items-center gap-2.5">
        <span class="neo-inset flex h-8 w-8 items-center justify-center text-[16px] text-amber icon">{ICON.lightning}</span>
        <div class="leading-tight">
          <div class="text-[14px] font-bold tracking-tight text-tprimary">DashKey</div>
          <div class="text-[10.5px] text-tmuted">{page ? `${page.name} · ${rows}×${cols}` : "Deck"}</div>
        </div>
      </div>
      <button
        class="neo-chip flex h-8 w-8 items-center justify-center text-[13px]"
        title="Ganti tema"
        aria-label="Ganti tema"
        onclick={() => themeCtx.toggle()}
      >
        {themeCtx.theme === "dark" ? "☀️" : "🌙"}
      </button>
    </div>

    <!-- NAVIGATION (selalu tampil) -->
    <div class="border-b border-border px-4 py-3">
      <label class="card-caption block mb-1.5" for="deck-profile">PROFILE</label>
      <select
        id="deck-profile"
        class="neo-inset w-full px-2.5 py-1.5 text-[12.5px] text-tprimary outline-none"
        value={config.active_profile}
        onchange={(e) => switchProfile(e.currentTarget.value)}
      >
        {#each config.profiles as p (p.profile_id)}
          <option value={p.profile_id}>{p.name}</option>
        {/each}
      </select>

      <div class="mt-3 flex items-center justify-between gap-2">
        <button
          class="neo-chip flex h-8 w-8 items-center justify-center text-[12px] text-tsecondary hover:text-tprimary"
          title="Page sebelumnya"
          aria-label="Page sebelumnya"
          disabled={pageIdx <= 0}
          onclick={goPrevPage}
        >
          ◀
        </button>
        <div class="flex min-w-0 flex-1 items-center justify-center gap-1.5">
          {#each profilePages as p, i (p.page_id)}
            <button
              class="h-2 w-2 rounded-full transition-all"
              class:bg-accent-soft={i === pageIdx}
              class:bg-surface-3={i !== pageIdx}
              title={p.name}
              onclick={() => selectPage(p.page_id)}
            ></button>
          {/each}
        </div>
        <button
          class="neo-chip flex h-8 w-8 items-center justify-center text-[12px] text-tsecondary hover:text-tprimary"
          title="Page berikutnya"
          aria-label="Page berikutnya"
          disabled={pageIdx < 0 || pageIdx >= profilePages.length - 1}
          onclick={goNextPage}
        >
          ▶
        </button>
      </div>

      <button
        class="neo-chip mt-3 flex w-full items-center justify-center gap-2 px-3 py-2 text-[12px] font-medium text-tsecondary hover:text-tprimary"
        onclick={createFolder}
      >
        <span class="icon text-[13px]">{ICON.folder}</span>
        <span>Create folder</span>
      </button>
    </div>

    <!-- APPS (accordion) -->
    <SidebarSection title="Apps" icon="monitor" count={apps.length} bind:open={appsOpen}>
      <div class="flex items-center gap-1.5">
        <div class="neo-inset flex min-w-0 flex-1 items-center gap-2 px-2.5 py-1.5">
          <span class="icon shrink-0 text-[12px] text-tmuted">{ICON.magnify}</span>
          <input
            bind:value={appSearch}
            class="min-w-0 flex-1 bg-transparent text-[12px] text-tprimary outline-none placeholder:text-tmuted"
            placeholder="Cari…"
          />
        </div>
        <button
          class="neo-chip flex h-7 w-7 shrink-0 items-center justify-center text-[11px] text-tsecondary hover:text-tprimary"
          title="Scan ulang"
          aria-label="Scan ulang"
          onclick={refreshApps}
        >
          {ICON.broadcast}
        </button>
      </div>

      <div class="mt-2 flex max-h-[300px] flex-col gap-1 overflow-y-auto pr-0.5">
        {#if appsBusy && apps.length === 0}
          <p class="px-2 py-4 text-center text-[11.5px] text-tmuted">Memindai aplikasi…</p>
        {:else if filteredApps.length === 0}
          <p class="px-2 py-4 text-center text-[11.5px] text-tmuted">
            {apps.length === 0 ? "Belum ada aplikasi terdeteksi." : "Tidak ada yang cocok."}
          </p>
        {:else}
          {#each filteredApps as app (app.name + app.target)}
            <div
              role="button"
              tabindex="0"
              class="neo-chip flex touch-none cursor-grab select-none items-center gap-2 px-2.5 py-1.5 active:cursor-grabbing"
              onpointerdown={(e) => rowPointerDown(e, { kind: "app", app, label: app.name })}
            >
              {#if app.icon_path && fileIconSrc(`file://${app.icon_path}`)}
                <img src={fileIconSrc(`file://${app.icon_path}`)} alt="" draggable="false" class="h-5 w-5 shrink-0 object-contain" />
              {:else}
                <span class="icon shrink-0 text-[14px] text-tsecondary">{ICON.monitor}</span>
              {/if}
              <span class="min-w-0 flex-1 truncate text-[12px] font-medium text-tprimary">{app.name}</span>
            </div>
          {/each}
        {/if}
      </div>
    </SidebarSection>

    <!-- SOUNDBOARD (accordion) -->
    <SidebarSection title="Soundboard" icon="music" count={sounds.length} bind:open={soundOpen}>
      <div class="flex max-h-[220px] flex-col gap-1 overflow-y-auto pr-0.5">
        {#if sounds.length === 0}
          <p class="px-2 py-3 text-[11.5px] text-tmuted">Belum ada file audio.</p>
        {:else}
          {#each sounds as file (file)}
            <div
              role="button"
              tabindex="0"
              class="neo-chip flex touch-none cursor-grab select-none items-center gap-2 px-2.5 py-1.5 active:cursor-grabbing"
              onpointerdown={(e) => rowPointerDown(e, { kind: "sound", sound: file, label: file })}
            >
              <button
                class="icon shrink-0 text-[13px] text-accent-soft hover:text-tprimary"
                title="Putar sekarang"
                aria-label={`Putar ${file}`}
                onclick={() => playSound(file)}
              >
                {ICON.play}
              </button>
              <span class="min-w-0 flex-1 truncate text-[12px] text-tsecondary">{file}</span>
            </div>
          {/each}
        {/if}
      </div>
      <div class="mt-2 flex gap-1.5">
        <button class="neo-chip flex-1 px-2 py-1.5 text-[11px] font-medium text-tsecondary hover:text-tprimary" onclick={() => openSoundsFolder().then(refreshSounds)}>
          Buka folder
        </button>
        <button class="neo-chip px-2 py-1.5 text-[11px] font-medium text-tsecondary hover:text-tprimary" onclick={refreshSounds} title="Muat ulang">
          ↻
        </button>
      </div>
    </SidebarSection>

    <!-- SYSTEM (accordion) -->
    <SidebarSection title="System" icon="gear" bind:open={systemOpen}>
      <div class="grid grid-cols-4 gap-1.5">
        {#each mediaActions as ma (ma.control)}
          <div
            role="button"
            tabindex="0"
            class="neo-chip flex touch-none cursor-grab select-none flex-col items-center gap-0.5 px-1 py-1.5 active:cursor-grabbing"
            title={`${ma.label} (seret ke slot untuk tombol)`}
            onpointerdown={(e) => rowPointerDown(e, { kind: "action", action: { action_type: "media_control", control: ma.control }, label: ma.label })}
            onclick={() => doQuickAction({ action_type: "media_control", control: ma.control })}
            onkeydown={(e) => e.key === "Enter" && doQuickAction({ action_type: "media_control", control: ma.control })}
          >
            <span class="text-[14px] leading-none">{ma.glyph}</span>
            <span class="max-w-full truncate text-[9.5px] text-tmuted">{ma.label}</span>
          </div>
        {/each}
      </div>
      <div class="mt-2 flex flex-col gap-1">
        {#each systemLinks as link (link.url)}
          <div
            role="button"
            tabindex="0"
            class="neo-chip flex touch-none cursor-grab select-none items-center gap-2 px-2.5 py-1.5 active:cursor-grabbing"
            onpointerdown={(e) => rowPointerDown(e, { kind: "action", action: { action_type: "open_url", target: link.url }, label: link.label })}
            onclick={() => doQuickAction({ action_type: "open_url", target: link.url })}
            onkeydown={(e) => e.key === "Enter" && doQuickAction({ action_type: "open_url", target: link.url })}
          >
            <span class="icon shrink-0 text-[13px] text-purple">{ICON.globe}</span>
            <span class="min-w-0 flex-1 truncate text-[12px] text-tsecondary">{link.label}</span>
          </div>
        {/each}
      </div>
      <p class="mt-2 text-[10.5px] leading-relaxed text-tmuted">
        Klik = jalankan sekarang. Seret ke slot grid = jadikan tombol.
      </p>
    </SidebarSection>

    <!-- Footer -->
    <div class="mt-auto border-t border-border p-3">
      <button
        class="neo-chip flex w-full items-center justify-center gap-2 px-3 py-2 text-[12px] font-medium text-tsecondary hover:text-tprimary"
        onclick={onOpenManage}
      >
        <span class="icon text-[13px]">{ICON.gear}</span>
        <span>Panel Manajemen</span>
      </button>
    </div>
  </aside>

  <!-- Deck utama -->
  <div class="flex min-w-0 flex-1 flex-col">
    <!-- Topbar ringkas -->
    <div class="flex items-center justify-between border-b border-border bg-surface-1/40 px-6 py-3">
      <div class="flex items-center gap-3">
        <span class="text-[13px] font-semibold text-tprimary">{page?.name ?? "Deck"}</span>
        <span class="text-[11.5px] text-tmuted">· {rows}×{cols}</span>
      </div>
      <div class="flex items-center gap-2 text-[12.5px]">
        <span class="inline-block h-2.5 w-2.5 rounded-full" class:bg-success={online} class:bg-tmuted={!online}></span>
        <span class:font-semibold={online} class:text-success={online} class:text-tmuted={!online}>
          {online ? `${status.connectionCount} device` : "Offline"}
        </span>
      </div>
    </div>

    <!-- Grid -->
    <div class="min-h-0 flex-1 overflow-y-auto p-6">
      <div class="flex justify-center">
        <div class="grid" style={`grid-template-columns: repeat(${cols}, minmax(0,1fr)); gap: 12px; width: min(100%, ${cols * 92}px);`}>
          {#each Array.from({ length: rows * cols }) as _, i (i)}
            {@const btnId = pageButtons[i]}
            {@const btn = btnId ? config.buttons[btnId] : undefined}
            <button
              data-slot={i}
              class={`flex h-[78px] flex-col items-center justify-center gap-1 rounded-xl px-1 text-center transition-all ${!btn || !btn.actions.length ? "neo-raised" : ""} ${selectedButtonId === btnId ? "ring-2 ring-accent-soft" : dropTarget === i ? "ring-2 ring-accent" : ""}`}
              style={btn ? `background: ${btn.color}22; box-shadow: inset 0 0 0 1px ${btn.color}55;` : ""}
              onclick={() => clickTile(btnId)}
              onpointerdown={btn ? (e) => tilePointerDown(e, btn.button_id, i) : undefined}
            >
              {#if btn}
                {#if btn.icon?.startsWith("file://") && fileIconSrc(btn.icon)}
                  <img src={fileIconSrc(btn.icon)} alt="" class="h-6 w-6 object-contain" />
                {:else if btn.icon || btn.label}
                  <span class="text-[21px] leading-none">{semanticGlyph(btn.icon)}</span>
                {/if}
                <span class="max-w-full truncate text-[10.5px] font-medium text-tprimary">
                  {btn.label || "…"}
                </span>
              {:else}
                <span class="icon text-[18px] text-tmuted">{ICON.plus}</span>
              {/if}
            </button>
          {/each}
        </div>
      </div>

      <!-- Config tombol -->
      {#if selectedButton}
        <div class="neo-raised mx-auto mt-6 max-w-[820px] p-5">
          <div class="flex items-center gap-3">
            <span class="h-7 w-7 rounded-md" style={`background: ${selectedButton.color}`}></span>
            <div>
              <div class="text-[15px] font-semibold text-tprimary">{selectedButton.label || "Tanpa label"}</div>
              <div class="text-[10px] text-tmuted">{selectedButton.button_id}</div>
            </div>
            <div class="ml-auto flex flex-wrap items-center gap-2">
              <button class="neo-chip px-3 py-1.5 text-[12px] font-medium text-tsecondary hover:text-tprimary" onclick={doTest}>Test</button>
              <button class="neo-chip px-3 py-1.5 text-[12px] font-medium text-tsecondary hover:text-tprimary" onclick={pickSound}>Pick Sound</button>
              <button class="neo-chip px-3 py-1.5 text-[12px] font-medium text-coral hover:text-tprimary" onclick={doDelete}>Delete</button>
            </div>
          </div>
          {#if testResult}
            <div class="mt-3 text-[12px] text-accent-soft">{testResult}</div>
          {/if}

          <div class="mt-4 grid grid-cols-2 gap-6">
            <div class="flex flex-col gap-3">
              <label class="card-caption block" for="deck-label">LABEL</label>
              <input
                id="deck-label"
                bind:value={labelDraft}
                class="neo-inset px-3 py-2 text-[13px] text-tprimary outline-none"
                placeholder="Nama tombol"
                onchange={commitLabel}
              />
              <div>
                <span class="card-caption block mb-1.5">COLOR</span>
                <div class="flex flex-wrap gap-2">
                  {#each BUTTON_COLORS as c (c)}
                    <button
                      class={`h-7 w-7 rounded-lg transition-transform hover:scale-110 ${selectedButton.color === c ? "ring-2 ring-tprimary" : ""}`}
                      style={`background: ${c}`}
                      aria-label={`Warna ${c}`}
                      onclick={() => setColor(c)}
                    ></button>
                  {/each}
                </div>
              </div>
              <div>
                <label class="card-caption block mb-1.5" for="deck-icon">ICON</label>
                <div class="flex flex-wrap items-center gap-1.5">
                  <select
                    id="deck-icon"
                    class="neo-inset max-w-[200px] px-2 py-1.5 text-[12.5px] text-tprimary outline-none"
                    value={selectedButton.icon ?? ""}
                    onchange={(e) => setIcon((e.currentTarget.value as string) || null)}
                  >
                    {#each ICON_OPTIONS as o (String(o.key))}
                      <option value={o.key ?? ""}>{o.label}</option>
                    {/each}
                  </select>
                  <button class="neo-chip px-2.5 py-1.5 text-[11.5px] text-tsecondary hover:text-tprimary" onclick={pickIconFile}>
                    File gambar…
                  </button>
                </div>
              </div>
            </div>

            <div class="flex flex-col gap-2">
              <div class="flex items-center justify-between">
                <span class="card-caption block">ACTIONS ({selectedButton.actions.length})</span>
                <button class="neo-chip px-2.5 py-1 text-[11.5px] font-medium text-accent-soft hover:text-tprimary" onclick={() => (showEditor = true)}>
                  Edit Aksi
                </button>
              </div>
              {#if selectedButton.actions.length === 0}
                <p class="text-[12px] text-tmuted">Belum ada aksi. Tambahkan untuk membuat tombol berfungsi.</p>
              {:else}
                <div class="flex flex-col gap-1.5">
                  {#each selectedButton.actions as action, i (i)}
                    <div class="flex items-center gap-2 rounded-lg bg-surface-0/60 px-3 py-2">
                      <span class="text-[11px] font-mono text-tmuted">{i + 1}</span>
                      <span class="min-w-0 flex-1 truncate text-[12.5px] text-tsecondary">{describeAction(action)}</span>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<!-- Ghost saat drag -->
{#if dragPayload && dragPos}
  <div
    class="pointer-events-none fixed z-[70] flex items-center gap-2 rounded-xl bg-accent/90 px-3 py-2 text-[12.5px] font-semibold text-btn-text shadow-lg"
    style={`left: ${dragPos.x + 14}px; top: ${dragPos.y + 14}px;`}
  >
    <span class="icon text-[14px]">{dragPayload.kind === "app" ? ICON.monitor : dragPayload.kind === "sound" ? ICON.music : dragPayload.kind === "action" ? ICON.lightning : ICON.squares_four}</span>
    <span class="max-w-[180px] truncate">{dragPayload.label}</span>
  </div>
{/if}

{#if showEditor && selectedButton}
  <ActionEditorModal
    button={selectedButton}
    onMutate={onMutate}
    onclose={() => (showEditor = false)}
  />
{/if}