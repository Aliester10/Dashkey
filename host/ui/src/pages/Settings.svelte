<script lang="ts">
  import { ICON } from "../lib/icons";
  import { getHostInfo, resetConfig, setAutostart } from "../lib/api";
  import { getConfirmCtx } from "../lib/confirm.svelte";
  import { getThemeCtx } from "../lib/theme.svelte";
  import type { HostInfo } from "../lib/types";
  import PageHeader from "../components/PageHeader.svelte";

  const confirm = getConfirmCtx();
  const themeCtx = getThemeCtx();

  let { onMutate }: { onMutate: () => Promise<void> } = $props();

  let info = $state<HostInfo | null>(null);
  let autostart = $state(false);
  let autostartBusy = $state(false);
  let showAdvanced = $state(false);

  $effect(() => {
    getHostInfo().then((i) => {
      info = i;
      autostart = i.autostart;
    });
  });

  async function toggleAutostart() {
    autostartBusy = true;
    try {
      await setAutostart(!autostart);
      autostart = !autostart;
    } finally {
      autostartBusy = false;
    }
  }

  async function doReset() {
    const ok = await confirm.requestConfirm({
      title: "Reset config?",
      message:
        "Semua profile, page, dan tombol akan dikembalikan ke pengaturan awal. Tindakan ini tidak bisa dibatalkan.",
      confirmLabel: "Ya, reset",
      danger: true,
    });
    if (ok) await resetConfig().then(onMutate);
  }
</script>

<div class="flex h-full flex-col gap-6 overflow-y-auto p-7">
  <PageHeader icon="gear" title="Settings" subtitle="Preferensi tampilan, runtime Host, dan keamanan lokal." />

  <div class="grid grid-cols-2 gap-5">
    <div class="neo-raised p-6">
      <div class="flex items-center gap-3">
        <span class="icon flex h-9 w-9 items-center justify-center rounded-xl bg-purple/15 text-[17px] text-purple">{ICON.gear}</span>
        <div class="text-[16px] font-semibold text-tprimary">Appearance</div>
      </div>
      <div class="mt-4 flex flex-col gap-3 text-[13px] text-tsecondary">
        <p>Tema gelap aktif untuk menjaga fokus saat streaming — atau pilih tema terang untuk siang hari.</p>
        <div class="flex gap-2">
          <button
            class={`neo-chip flex-1 px-4 py-2.5 text-[13px] font-medium ${themeCtx.theme === "dark" ? "ring-2 ring-accent-soft text-accent-soft" : "text-tsecondary hover:text-tprimary"}`}
            onclick={() => themeCtx.set("dark")}
          >
            🌙 Gelap
          </button>
          <button
            class={`neo-chip flex-1 px-4 py-2.5 text-[13px] font-medium ${themeCtx.theme === "light" ? "ring-2 ring-accent-soft text-accent-soft" : "text-tsecondary hover:text-tprimary"}`}
            onclick={() => themeCtx.set("light")}
          >
            ☀️ Terang
          </button>
        </div>
        <p class="text-[12px] text-tmuted">
          Pilihan tersimpan otomatis; mode awal mengikuti tema sistem.
        </p>
      </div>
    </div>

    <div class="neo-raised p-6">
      <div class="flex items-center gap-3">
        <span class="icon flex h-9 w-9 items-center justify-center rounded-xl bg-success/15 text-[17px] text-success">{ICON.plugs}</span>
        <div class="text-[16px] font-semibold text-tprimary">Host runtime</div>
      </div>
      <div class="mt-4 flex flex-col gap-2.5 text-[13px]">
        <div class="flex justify-between">
          <span class="text-tmuted">Host name</span>
          <span class="font-mono text-tprimary">{info?.hostName ?? "…"}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-tmuted">LAN address</span>
          <span class="font-mono text-tprimary">{info?.hostIp ?? "…"}:{info?.port ?? "…"}</span>
        </div>
        <div class="flex justify-between gap-4">
          <span class="shrink-0 text-tmuted">Config</span>
          <span class="truncate font-mono text-[11.5px] text-tsecondary">{info?.dataDir ?? "…"}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-tmuted">Version</span>
          <span class="font-mono text-tprimary">v{info?.version ?? "…"}</span>
        </div>
        <label class="mt-1 flex items-center justify-between rounded-lg bg-surface-0/60 px-3 py-2">
          <span>Launch Host on startup</span>
          <input
            type="checkbox"
            checked={autostart}
            disabled={autostartBusy}
            class="accent-accent"
            onchange={toggleAutostart}
          />
        </label>
      </div>
    </div>
  </div>

  <div class="neo-raised p-6">
    <div class="flex items-center gap-3">
      <span class="icon flex h-9 w-9 items-center justify-center rounded-xl bg-amber/15 text-[17px] text-amber">{ICON.lightning}</span>
      <div>
        <div class="text-[16px] font-semibold text-tprimary">Safety & advanced controls</div>
        <div class="mt-1 text-[12px] leading-relaxed text-tmuted">
          Komunikasi hanya berjalan di jaringan lokal. Aksi membutuhkan device pairing yang valid.
        </div>
      </div>
    </div>

    <button
      class="mt-3 neo-chip px-3 py-1.5 text-[12px] font-medium text-tsecondary hover:text-tprimary"
      onclick={() => (showAdvanced = !showAdvanced)}
    >
      {showAdvanced ? "Sembunyikan" : "Tampilkan"} advanced controls
    </button>

    {#if showAdvanced}
      <div class="mt-4 flex flex-col gap-3">
        <div class="flex gap-3">
          <button class="neo-chip px-4 py-2 text-[13px] font-medium text-coral hover:text-tprimary" onclick={doReset}>
            Reset config ke default
          </button>
        </div>
        <div class="flex flex-col gap-1 font-mono text-[11.5px] text-tmuted">
          <span>DASHKEY_PORT=&lt;port&gt;</span>
          <span>DASHKEY_NO_GUI=1</span>
        </div>
      </div>
    {/if}
  </div>
</div>
