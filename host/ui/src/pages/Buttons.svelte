<script lang="ts">
  import { onDestroy, onMount, untrack } from "svelte";
  import { ICON } from "../lib/icons";
  import {
    addPlaySound,
    createAppButton,
    createButton,
    deleteButton,
    fileIconSrc,
    moveButton,
    scanApps,
    setActivePage,
    setButtonIconFile,
    testButton,
    updateButton,
  } from "../lib/api";
  import { BUTTON_COLORS, describeAction, ICON_OPTIONS } from "../lib/constants";
  import { getConfirmCtx } from "../lib/confirm.svelte";
  import type { Button, Config, DetectedApp } from "../lib/types";
  import { open } from "@tauri-apps/plugin-dialog";
  import Modal from "../components/Modal.svelte";
  import ActionEditorModal from "../components/ActionEditorModal.svelte";

  const confirm = getConfirmCtx();

  let {
    config,
    onMutate,
  }: {
    config: Config;
    onMutate: () => Promise<void>;
  } = $props();

  let selectedPage = $state(untrack(() => config.active_page));
  let selectedButtonId = $state<string | null>(null);
  let labelDraft = $state("");
  let showAppPicker = $state(false);
  let showEditor = $state(false);
  let apps = $state<DetectedApp[]>([]);
  let appSearch = $state("");
  let busy = $state(false);
  let testResult = $state("");

  // Drag & drop tombol antar slot grid (pointer events — stabil di WebView2).
  let dragTile = $state<{ buttonId: string; fromIdx: number; startX: number; startY: number } | null>(null);
  let dragPayload = $state<{ buttonId: string; fromIdx: number; label: string } | null>(null);
  let dragPos = $state<{ x: number; y: number } | null>(null);
  let dropTarget = $state<number | null>(null);
  let suppressClick = $state(false);

  onMount(() => {
    window.addEventListener("pointermove", windowPointerMove);
    window.addEventListener("pointerup", windowPointerUp);
    window.addEventListener("pointercancel", endDrag);
  });

  onDestroy(() => {
    window.removeEventListener("pointermove", windowPointerMove);
    window.removeEventListener("pointerup", windowPointerUp);
    window.removeEventListener("pointercancel", endDrag);
  });

  function tilePointerDown(e: PointerEvent, buttonId: string, slotIdx: number) {
    if (e.button !== 0) return;
    dragTile = { buttonId, fromIdx: slotIdx, startX: e.clientX, startY: e.clientY };
    suppressClick = false;
  }

  function slotAt(x: number, y: number): number | null {
    const el = document.elementFromPoint(x, y);
    const tile = el?.closest("[data-slot]") as HTMLElement | null;
    if (!tile) return null;
    const idx = Number(tile.dataset.slot);
    return Number.isInteger(idx) ? idx : null;
  }

  function windowPointerMove(e: PointerEvent) {
    if (!dragPayload) {
      if (dragTile && Math.hypot(e.clientX - dragTile.startX, e.clientY - dragTile.startY) > 5) {
        const btn = config.buttons[dragTile.buttonId];
        dragPayload = {
          buttonId: dragTile.buttonId,
          fromIdx: dragTile.fromIdx,
          label: btn?.label || "…",
        };
        dragPos = { x: e.clientX, y: e.clientY };
        dropTarget = null;
      }
      return;
    }
    dragPos = { x: e.clientX, y: e.clientY };
    dropTarget = slotAt(e.clientX, e.clientY);
  }

  function endDrag() {
    dragPayload = null;
    dragPos = null;
    dropTarget = null;
    dragTile = null;
  }

  function windowPointerUp() {
    if (!dragPayload) {
      dragTile = null;
      return;
    }
    const payload = dragPayload;
    const idx = dropTarget;
    endDrag();
    suppressClick = true;
    if (idx !== null && idx !== payload.fromIdx) {
      mutate(() => moveButton(selectedPage, payload.fromIdx, idx));
    }
  }

  const pages = $derived(Object.values(config.pages).sort((a, b) => a.page_id.localeCompare(b.page_id)));
  const page = $derived(config.pages[selectedPage]);
  const pageButtons = $derived(page?.buttons ?? []);
  const rows = $derived(page?.grid_size.rows ?? 4);
  const cols = $derived(page?.grid_size.cols ?? 4);
  const selectedButton = $derived(selectedButtonId ? config.buttons[selectedButtonId] : undefined);

  $effect(() => {
    if (selectedButton) labelDraft = selectedButton.label;
  });

  function semanticGlyph(icon: string | null | undefined): string {
    if (!icon || icon.startsWith("file://")) return "⚡";
    const opt = ICON_OPTIONS.find((o) => o.key === icon);
    if (!opt?.label) return "⚡";
    return opt.label.split(" ")[0] ?? "⚡";
  }

  async function mutate(fn: () => Promise<unknown>) {
    busy = true;
    try {
      await fn();
      await onMutate();
    } finally {
      busy = false;
    }
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

  function clickTile(buttonId: string | null) {
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

  async function rescanApps() {
    apps = await scanApps();
    // Backend bisa mengisi icon tombol app yang sudah ada → ambil ulang config.
    await onMutate();
  }

  function openAppPicker() {
    showAppPicker = true;
    rescanApps();
  }

  async function addApp(app: DetectedApp) {
    await mutate(() => createAppButton(selectedPage, app));
    showAppPicker = false;
  }

  const filteredApps = $derived(
    apps.filter((a) => !appSearch || a.name.toLowerCase().includes(appSearch.toLowerCase())),
  );
</script>

<div class="flex h-full flex-col overflow-hidden">
  <!-- Page nav + app picker -->
  <div class="flex items-center gap-3 border-b border-border bg-surface-1/40 px-6 py-3">
    <span class="icon text-[16px] text-accent-soft">{ICON.grid_four}</span>
    <div class="flex items-center gap-1.5">
      {#each pages as p (p.page_id)}
        <button
          class={`rounded-lg px-3 py-1.5 text-[12.5px] font-medium transition-colors ${selectedPage === p.page_id ? "bg-accent/15 text-accent-soft" : "text-tsecondary hover:text-tprimary"}`}
          onclick={() => selectPage(p.page_id)}
        >
          {p.name}
        </button>
      {/each}
    </div>
    <span class="text-[11.5px] text-tmuted">·</span>
    <span class="text-[12px] text-tmuted">{rows}×{cols}</span>
    <div class="ml-auto flex items-center gap-2">
      <button class="neo-chip px-3 py-1.5 text-[12.5px] font-medium text-tsecondary hover:text-tprimary" onclick={openAppPicker}>
        + App
      </button>
    </div>
  </div>

  <!-- Grid + config -->
  <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-6">
    <!-- Grid -->
    <div class="flex justify-center">
      <div class="grid" style={`grid-template-columns: repeat(${cols}, minmax(0,1fr)); gap: 10px; width: min(100%, ${cols * 86}px);`}>
        {#each Array.from({ length: rows * cols }) as _, i (i)}
          {@const btnId = pageButtons[i]}
          {@const btn = btnId ? config.buttons[btnId] : undefined}
          <button
            data-slot={i}
            class={`flex h-[74px] flex-col items-center justify-center gap-1 rounded-xl px-1 text-center transition-all ${!btn || !btn.actions.length ? "neo-raised" : ""} ${selectedButtonId === btnId ? "ring-2 ring-accent-soft" : dropTarget === i ? "ring-2 ring-accent" : ""}`}
            class:cursor-grab={!!btn}
            style={btn ? `background: ${btn.color}22; box-shadow: inset 0 0 0 1px ${btn.color}55;` : ""}
            onclick={() => clickTile(btnId)}
            onpointerdown={btn ? (e) => tilePointerDown(e, btn.button_id, i) : undefined}
          >
            {#if btn}
              {#if btn.icon?.startsWith("file://") && fileIconSrc(btn.icon)}
                <img src={fileIconSrc(btn.icon)} alt="" class="h-6 w-6 object-contain" />
              {:else if btn.icon || btn.label}
                <span class="text-[20px] leading-none">{semanticGlyph(btn.icon)}</span>
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

    <!-- Selected button config -->
    {#if selectedButton}
      <div class="neo-raised p-5">
        <div class="flex items-center gap-3">
          <span class="h-7 w-7 rounded-md" style={`background: ${selectedButton.color}`}></span>
          <div>
            <div class="text-[15px] font-semibold text-tprimary">{selectedButton.label || "Tanpa label"}</div>
            <div class="text-[10px] text-tmuted">{selectedButton.button_id}</div>
          </div>
          <div class="ml-auto flex flex-wrap items-center gap-2">
            <button class="neo-chip px-3 py-1.5 text-[12px] font-medium text-tsecondary hover:text-tprimary" onclick={doTest}>
              Test
            </button>
            <button class="neo-chip px-3 py-1.5 text-[12px] font-medium text-tsecondary hover:text-tprimary" onclick={pickSound}>
              Pick Sound
            </button>
            <button
              class="neo-chip px-3 py-1.5 text-[12px] font-medium text-coral hover:text-tprimary"
              onclick={doDelete}
            >
              Delete
            </button>
          </div>
        </div>
        {#if testResult}
          <div class="mt-3 text-[12px] text-accent-soft">{testResult}</div>
        {/if}

        <div class="mt-4 grid grid-cols-2 gap-6">
          <div class="flex flex-col gap-3">
            <label class="card-caption block" for="btn-label">LABEL</label>
            <input
              id="btn-label"
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
              <label class="card-caption block mb-1.5" for="btn-icon">ICON</label>
              <div class="flex flex-wrap items-center gap-1.5">
                <select
                  id="btn-icon"
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
              <button
                class="neo-chip px-2.5 py-1 text-[11.5px] font-medium text-accent-soft hover:text-tprimary"
                onclick={() => (showEditor = true)}
              >
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
                    <span class="min-w-0 flex-1 truncate text-[12.5px] text-tsecondary">
                      {describeAction(action)}
                    </span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </div>
    {:else}
      <div class="neo-raised flex flex-col items-center gap-3 p-8 text-center">
        <p class="text-[13px] text-tmuted">Pilih tombol untuk mengatur label, warna, ikon, dan aksinya.</p>
        <button class="btn-primary flex items-center gap-2 px-4 py-2" onclick={() => addKeyAt()}>
          <span class="icon text-[15px]">{ICON.plus}</span>
          <span>Add Key</span>
        </button>
      </div>
    {/if}
  </div>
</div>

<!-- App picker modal -->
{#if showAppPicker}
  <Modal title="Pick Installed App" width={520} onclose={() => (showAppPicker = false)}>
    <div class="flex items-center justify-between">
      <span class="text-[13px] font-medium text-tsecondary">{apps.length} apps detected</span>
      <button class="neo-chip px-2.5 py-1 text-[11.5px] text-tsecondary hover:text-tprimary" onclick={rescanApps}>
        Rescan
      </button>
    </div>
    <div class="mt-3 flex items-center gap-2">
      <input
        bind:value={appSearch}
        class="neo-inset min-w-0 flex-1 px-3 py-2 text-[13px] text-tprimary outline-none"
        placeholder="Cari nama aplikasi…"
      />
    </div>
    <div class="divider my-3"></div>
    <div class="flex max-h-[340px] flex-col gap-1.5 overflow-y-auto">
      {#if filteredApps.length === 0}
        <p class="py-4 text-center text-[12.5px] text-tmuted">Tidak ada aplikasi yang cocok.</p>
      {:else}
        {#each filteredApps as app (app.name + app.target)}
          <div class="flex items-center gap-2 rounded-lg px-2 py-1.5 hover:bg-hover">
            <button class="neo-chip px-2 py-0.5 text-[14px] font-bold text-success" onclick={() => addApp(app)}>+</button>
            <span class="min-w-0 flex-1 truncate text-[13px] text-tprimary">{app.name}</span>
            <span class="max-w-[40%] truncate text-[11px] text-tmuted">{app.target}</span>
          </div>
        {/each}
      {/if}
    </div>
  </Modal>
{/if}

{#if showEditor && selectedButton}
  <ActionEditorModal
    button={selectedButton}
    onMutate={onMutate}
    onclose={() => (showEditor = false)}
  />
{/if}

<!-- Ghost saat drag -->
{#if dragPayload && dragPos}
  <div
    class="pointer-events-none fixed z-[70] flex items-center gap-2 rounded-xl bg-accent/90 px-3 py-2 text-[12.5px] font-semibold text-btn-text shadow-lg"
    style={`left: ${dragPos.x + 14}px; top: ${dragPos.y + 14}px;`}
  >
    <span class="icon text-[14px]">{ICON.squares_four}</span>
    <span class="max-w-[180px] truncate">{dragPayload.label}</span>
  </div>
{/if}