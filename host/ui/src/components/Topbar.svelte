<script lang="ts">
  import { ICON } from "../lib/icons";
  import { getThemeCtx } from "../lib/theme.svelte";

  const themeCtx = getThemeCtx();

  let {
    title,
    subtitle,
    online,
    deviceCount,
    hostIp,
    port,
    hostName,
  }: {
    title: string;
    subtitle: string;
    online: boolean;
    deviceCount: number;
    hostIp: string;
    hostName: string;
    port: number;
  } = $props();
</script>

<header class="flex items-center justify-between border-b border-border bg-surface-1/60 px-6 py-4">
  <div>
    <h1 class="text-[20px] font-bold tracking-tight text-tprimary">{title}</h1>
    <p class="mt-0.5 text-[12.5px] text-tsecondary">{subtitle}</p>
  </div>

  <div class="flex items-center gap-3">
    <div class="flex items-center gap-2 text-[12.5px]">
      <span
        class="inline-block h-2.5 w-2.5 rounded-full"
        class:bg-success={online}
        class:bg-tmuted={!online}
      ></span>
      <span class:font-semibold={online} class:text-success={online} class:text-tmuted={!online}>
        {online ? `${deviceCount} device online` : "Belum ada device"}
      </span>
    </div>

    <button
      class="neo-chip flex h-9 w-9 items-center justify-center text-[15px]"
      title={themeCtx.theme === "dark" ? "Ganti ke tema terang" : "Ganti ke tema gelap"}
      aria-label="Ganti tema"
      onclick={() => themeCtx.toggle()}
    >
      {themeCtx.theme === "dark" ? "☀️" : "🌙"}
    </button>

    <div class="neo-inset flex items-center gap-2 px-3 py-1.5 text-[12px] text-tsecondary">
      <span class="icon text-[15px] text-accent-soft">{ICON.wifi}</span>
      <span class="font-mono">{hostIp}:{port}</span>
      <span class="text-tmuted">·</span>
      <span>{hostName}</span>
    </div>
  </div>
</header>