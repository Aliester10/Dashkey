<script lang="ts">
  import { ICON } from "../lib/icons";
  import type { Config, StatusPayload } from "../lib/types";
  import { broadcastConfig, formatDuration } from "../lib/api";
  import StatCard from "./StatCard.svelte";
  import Panel from "./Panel.svelte";

  let {
    config,
    status,
    onNavigate,
  }: {
    config: Config;
    status: StatusPayload;
    onNavigate: (tab: string) => void;
  } = $props();

  const online = $derived(status.connectionCount > 0);
  const buttons = $derived(Object.keys(config.buttons).length);
  const pages = $derived(Object.keys(config.pages).length);
  const profiles = $derived(config.profiles.length);

  const quickStart = [
    { icon: ICON.qr_code, label: "Pair device baru", tab: "pairing" },
    { icon: ICON.squares_four, label: "Kelola tombol", tab: "buttons" },
    { icon: ICON.plugs, label: "Integrasi OBS & soundboard", tab: "integrations" },
  ] as const;
</script>

<div class="flex h-full flex-col gap-6 overflow-y-auto p-7">
  <!-- Stat cards -->
  <div class="grid grid-cols-2 gap-4 xl:grid-cols-4">
    <StatCard
      icon="plugs"
      label="DEVICE"
      value={String(status.connectionCount)}
      caption="→ Devices"
      accent="success"
      onclick={() => onNavigate("devices")}
    />
    <StatCard
      icon="user_circle"
      label="PROFILE"
      value={String(profiles)}
      caption="→ Profiles"
      accent="purple"
      onclick={() => onNavigate("profiles")}
    />
    <StatCard
      icon="stack"
      label="PAGE"
      value={String(pages)}
      caption="→ Buttons"
      accent="accent"
      onclick={() => onNavigate("buttons")}
    />
    <StatCard
      icon="squares_four"
      label="BUTTON"
      value={String(buttons)}
      caption="→ Buttons"
      accent="muted"
      onclick={() => onNavigate("buttons")}
    />
  </div>

  <!-- Quick start + Activity -->
  <div class="grid grid-cols-1 gap-4 xl:grid-cols-2">
    <Panel title="Quick Start">
      <p class="card-caption leading-relaxed">
        Pairing HP, lalu tambahkan aplikasi sebagai shortcut.
      </p>
      <div class="mt-4 flex flex-col gap-3">
        {#each quickStart as qs}
          <button
            class="neo-chip flex w-full cursor-pointer items-center gap-3 px-4 py-3 text-left text-[13px] font-medium text-tprimary"
            onclick={() => onNavigate(qs.tab)}
          >
            <span class="icon text-[18px] text-accent-soft">{qs.icon}</span>
            <span>{qs.label}</span>
          </button>
        {/each}
      </div>
    </Panel>

    <Panel title="Recent Activity">
      {#if status.activity.length === 0}
        <p class="card-caption">Belum ada aktivitas.</p>
      {:else}
        <ul class="flex flex-col gap-3">
          {#each [...status.activity].reverse().slice(0, 6) as item, i (i)}
            <li class="flex items-start gap-2.5 text-[12.5px] text-tsecondary">
              <span class="icon mt-0.5 text-[13px] text-accent-soft">{ICON.activity}</span>
              <span class="leading-snug">{item}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </Panel>
  </div>

  <!-- Status bar -->
  <div class="neo-inset flex items-center justify-between px-5 py-3.5">
    <div class="flex items-center gap-4 text-[13px]">
      <span class:font-semibold={online} class:text-success={online} class:text-tmuted={!online}>
        {online ? "Host berjalan normal" : "Menunggu koneksi device"}
      </span>
      <span class="text-[12.5px] text-tmuted">
        {status.hostIp}:{status.port} · uptime {formatDuration(status.uptimeSecs)}
      </span>
    </div>
    <button class="btn-primary flex items-center gap-2 px-4 py-2" onclick={() => broadcastConfig()}>
      <span class="icon text-[15px]">{ICON.broadcast}</span>
      <span>Broadcast config</span>
    </button>
  </div>
</div>
