<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Sidebar, { type NavItem } from "./components/Sidebar.svelte";
  import Topbar from "./components/Topbar.svelte";
  import Dashboard from "./components/Dashboard.svelte";
  import ConfirmModal from "./components/ConfirmModal.svelte";
  import Pairing from "./pages/Pairing.svelte";
  import Devices from "./pages/Devices.svelte";
  import Buttons from "./pages/Buttons.svelte";
  import Profiles from "./pages/Profiles.svelte";
  import Integrations from "./pages/Integrations.svelte";
  import Activity from "./pages/Activity.svelte";
  import Settings from "./pages/Settings.svelte";
  import { getSnapshot, getStatus } from "./lib/api";
  import { createConfirmCtx, setConfirmCtx } from "./lib/confirm.svelte";
  import type { Config, StatusPayload } from "./lib/types";

  const confirmCtx = createConfirmCtx();
  setConfirmCtx(confirmCtx);

  const nav: NavItem[] = [
    { id: "dashboard", label: "Dashboard", icon: "squares_four" },
    { id: "buttons", label: "Buttons", icon: "grid_four" },
    { id: "profiles", label: "Profiles", icon: "user_circle" },
    { id: "pairing", label: "Pairing", icon: "qr_code" },
    { id: "devices", label: "Devices", icon: "devices" },
    { id: "integrations", label: "Integrations", icon: "plugs_connected" },
    { id: "activity", label: "Activity", icon: "list_bullets" },
    { id: "settings", label: "Settings", icon: "gear" },
  ];

  let active = $state("dashboard");
  let config = $state<Config | null>(null);
  let status = $state<StatusPayload | null>(null);
  let error = $state<string | null>(null);

  const online = $derived((status?.connectionCount ?? 0) > 0);
  const title = $derived(nav.find((n) => n.id === active)?.label ?? "DashKey");

  async function refreshSnapshot() {
    try {
      config = await getSnapshot();
    } catch (e) {
      error = String(e);
    }
  }

  async function refreshStatus() {
    try {
      status = await getStatus();
    } catch (e) {
      error = String(e);
    }
  }

  async function refresh() {
    await Promise.all([refreshSnapshot(), refreshStatus()]);
  }

  let statusTimer: ReturnType<typeof setInterval> | undefined;
  let snapshotTimer: ReturnType<typeof setInterval> | undefined;
  const unlisteners: (() => void)[] = [];

  onMount(() => {
    refresh();
    statusTimer = setInterval(refreshStatus, 500);
    snapshotTimer = setInterval(refreshSnapshot, 1000);
    listen("config_synced", refreshSnapshot).then((u) => unlisteners.push(u));
    listen("device_status", refreshStatus).then((u) => unlisteners.push(u));
  });

  onDestroy(() => {
    if (statusTimer) clearInterval(statusTimer);
    if (snapshotTimer) clearInterval(snapshotTimer);
    unlisteners.forEach((u) => u());
  });
</script>

{#if error && !config}
  <div class="flex h-full items-center justify-center">
    <div class="neo-raised p-6 text-[13px] text-coral">Gagal memuat data Host: {error}</div>
  </div>
{:else if !config || !status}
  <div class="flex h-full items-center justify-center text-[13px] text-tmuted">Memuat…</div>
{:else}
  <div class="flex h-full">
    <Sidebar items={nav} {active} {online} onselect={(id) => (active = id)} />
    <div class="flex min-w-0 flex-1 flex-col">
      <Topbar
        {title}
        subtitle={active === "dashboard" ? "Pusat kendali DashKey — device, tombol, dan integrasi PC." : ""}
        {online}
        deviceCount={status.connectionCount}
        hostIp={status.hostIp}
        hostName={status.hostName}
        port={status.port}
      />
      <main class="min-h-0 flex-1 overflow-hidden">
        {#if active === "dashboard"}
          <Dashboard {config} {status} onNavigate={(id) => (active = id)} />
        {:else if active === "buttons"}
          <Buttons {config} onMutate={refresh} />
        {:else if active === "profiles"}
          <Profiles {config} onMutate={refresh} />
        {:else if active === "pairing"}
          <Pairing />
        {:else if active === "devices"}
          <Devices />
        {:else if active === "integrations"}
          <Integrations {config} onMutate={refresh} />
        {:else if active === "activity"}
          <Activity {status} onMutate={refresh} />
        {:else if active === "settings"}
          <Settings onMutate={refresh} />
        {/if}
      </main>
    </div>
  </div>
{/if}

{#if confirmCtx.pending}
  <ConfirmModal ctx={confirmCtx} />
{/if}
