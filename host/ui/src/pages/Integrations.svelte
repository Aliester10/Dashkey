<script lang="ts">
  import { untrack } from "svelte";
  import { ICON } from "../lib/icons";
  import {
    importSfx,
    listSounds,
    openSoundsFolder,
    playSound,
    scanApps,
    setObsSettings,
    testObs,
  } from "../lib/api";
  import type { Config } from "../lib/types";
  import PageHeader from "../components/PageHeader.svelte";

  let {
    config,
    onMutate,
  }: { config: Config; onMutate: () => Promise<void> } = $props();

  let obsHost = $state(untrack(() => config.obs.host));
  let obsPort = $state(untrack(() => String(config.obs.port)));
  let obsPassword = $state(untrack(() => config.obs.password ?? ""));
  let obsStatus = $state<string | null>(null);
  let obsBusy = $state(false);

  let sounds = $state<string[]>([]);
  let sfxInput = $state("");
  let sfxMsg = $state<string | null>(null);
  let appCount = $state(0);

  const soundButtons = $derived(
    Object.values(config.buttons).filter((b) =>
      b.actions.some((a) => a.action_type === "play_sound"),
    ).length,
  );

  const obsState = $derived.by(() => {
    if (obsStatus && obsStatus.startsWith("OBS")) return { label: "ONLINE", cls: "text-success" };
    if (obsStatus && obsStatus.startsWith("gagal")) return { label: "ERROR", cls: "text-coral" };
    if (obsStatus) return { label: obsStatus, cls: "text-amber" };
    return { label: "NOT TESTED", cls: "text-tmuted" };
  });

  async function refreshSounds() {
    sounds = await listSounds();
  }

  async function doSaveObs() {
    const port = parseInt(obsPort, 10) || 4455;
    await setObsSettings(obsHost.trim(), port, obsPassword);
    await onMutate();
  }

  async function doTestObs() {
    obsBusy = true;
    obsStatus = "Menghubungi…";
    try {
      obsStatus = await testObs();
    } catch (e) {
      obsStatus = `gagal: ${e}`;
    } finally {
      obsBusy = false;
    }
  }

  async function doPlay(file: string) {
    try {
      await playSound(file);
    } catch (e) {
      sfxMsg = String(e);
    }
  }

  async function doImportSfx() {
    if (!sfxInput.trim()) return;
    sfxMsg = null;
    try {
      const imp = await importSfx(sfxInput.trim());
      sfxMsg = `SFX diimpor: ${imp.file_name}`;
      sfxInput = "";
      await refreshSounds();
    } catch (e) {
      sfxMsg = `Gagal impor: ${e}`;
    }
  }

  async function doScanApps() {
    const apps = await scanApps();
    appCount = apps.length;
  }

  // init
  $effect(() => {
    refreshSounds();
    scanApps().then((a) => (appCount = a.length));
  });
</script>

<div class="flex h-full flex-col gap-6 overflow-y-auto p-7">
  <PageHeader
    icon="plugs_connected"
    title="Integrations"
    subtitle="Hubungkan DashKey dengan workflow favorit Anda."
  />

  <div class="grid grid-cols-2 gap-5">
    <!-- OBS -->
    <div class="neo-raised p-6">
      <div class="flex items-center gap-3">
        <span class="icon flex h-10 w-10 items-center justify-center rounded-xl bg-purple/15 text-[18px] text-purple">{ICON.plugs}</span>
        <div>
          <div class="text-[16px] font-semibold text-tprimary">OBS Studio</div>
          <div class="text-[11px] text-tmuted">Scene, mute, stream, recording</div>
        </div>
        <div class="ml-auto">
          <span class="neo-inset px-3 py-1 text-[11px] font-semibold {obsState.cls}">{obsState.label}</span>
        </div>
      </div>
      <div class="mt-4 flex gap-3">
        <label class="flex-1">
          <span class="card-caption block mb-1">HOST</span>
          <input bind:value={obsHost} class="neo-inset w-full px-3 py-2 text-[13px] text-tprimary outline-none" placeholder="localhost" />
        </label>
        <label class="w-24">
          <span class="card-caption block mb-1">PORT</span>
          <input bind:value={obsPort} class="neo-inset w-full px-3 py-2 text-[13px] text-tprimary outline-none" placeholder="4455" />
        </label>
      </div>
      <label class="mt-3 block">
        <span class="card-caption block mb-1">PASSWORD</span>
        <input type="password" bind:value={obsPassword} class="neo-inset w-full px-3 py-2 text-[13px] text-tprimary outline-none" placeholder="OBS WebSocket password" />
      </label>
      <div class="mt-4 flex gap-3">
        <button class="neo-chip px-4 py-2 text-[13px] font-medium text-tsecondary hover:text-tprimary" onclick={doSaveObs}>
          Simpan
        </button>
        <button class="btn-primary px-4 py-2" onclick={doTestObs} disabled={obsBusy}>
          {obsBusy ? "Menghubungi…" : "Test connection"}
        </button>
      </div>
    </div>

    <!-- Soundboard -->
    <div class="neo-raised p-6">
      <div class="flex items-center gap-3">
        <span class="icon flex h-10 w-10 items-center justify-center rounded-xl bg-amber/15 text-[18px] text-amber">{ICON.music}</span>
        <div>
          <div class="text-[16px] font-semibold text-tprimary">Soundboard</div>
          <div class="text-[11px] text-tmuted">File audio lokal dan SFX</div>
        </div>
      </div>
      <div class="mt-4 text-[13px] text-tsecondary">
        {soundButtons} button · {sounds.length} file audio
      </div>
      <div class="mt-2 flex max-h-[110px] flex-col gap-1 overflow-y-auto pr-1">
        {#if sounds.length === 0}
          <p class="text-[12px] text-tmuted">Belum ada file audio.</p>
        {:else}
          {#each sounds as file (file)}
            <div class="flex items-center gap-2 rounded-lg px-2 py-1 hover:bg-white/5">
              <button class="icon text-[14px] text-accent-soft hover:text-tprimary" onclick={() => doPlay(file)}>{ICON.play}</button>
              <span class="min-w-0 flex-1 truncate text-[12.5px] text-tsecondary">{file}</span>
            </div>
          {/each}
        {/if}
      </div>
      <div class="mt-3 flex items-center gap-2">
        <input
          bind:value={sfxInput}
          class="neo-inset min-w-0 flex-1 px-3 py-2 text-[12.5px] text-tprimary outline-none"
          placeholder="URL / iframe myinstants.com…"
          onkeydown={(e) => e.key === "Enter" && doImportSfx()}
        />
        <button class="neo-chip px-3 py-2 text-[12px] font-medium text-accent-soft hover:text-tprimary" onclick={doImportSfx}>
          Impor SFX
        </button>
      </div>
      {#if sfxMsg}
        <p class="mt-2 text-[12px] text-accent-soft">{sfxMsg}</p>
      {/if}
      <div class="mt-3">
        <button class="neo-chip px-3 py-1.5 text-[12px] font-medium text-tsecondary hover:text-tprimary" onclick={openSoundsFolder}>
          Buka folder sounds
        </button>
      </div>
    </div>
  </div>

  <div class="grid grid-cols-2 gap-5">
    <!-- App launcher -->
    <div class="neo-raised p-6">
      <div class="flex items-center gap-3">
        <span class="icon flex h-9 w-9 items-center justify-center rounded-xl bg-accent/15 text-[17px] text-accent-soft">{ICON.squares_four}</span>
        <div>
          <div class="text-[15px] font-semibold text-tprimary">Application launcher</div>
          <div class="mt-1 text-[13px] text-tmuted">{appCount} aplikasi terdeteksi</div>
        </div>
      </div>
      <button class="mt-3 neo-chip px-3 py-1.5 text-[12px] font-medium text-tsecondary hover:text-tprimary" onclick={doScanApps}>
        Scan ulang aplikasi
      </button>
    </div>

    <!-- Automation -->
    <div class="neo-raised p-6">
      <div class="flex items-center gap-3">
        <span class="icon flex h-9 w-9 items-center justify-center rounded-xl bg-purple/15 text-[17px] text-purple">{ICON.gear}</span>
        <div>
          <div class="text-[15px] font-semibold text-tprimary">System automation</div>
          <div class="mt-1 text-[13px] text-tmuted">Keyboard, shell command, URL, media keys, dan aksi OBS.</div>
        </div>
      </div>
      <button class="mt-3 neo-chip px-3 py-1.5 text-[12px] font-medium text-tsecondary hover:text-tprimary" onclick={() => (sfxMsg = "Lihat daftar aksi di Buttons → Edit Aksi")}>
        Lihat aksi didukung
      </button>
    </div>
  </div>
</div>
