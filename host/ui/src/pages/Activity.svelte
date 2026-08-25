<script lang="ts">
  import { ICON } from "../lib/icons";
  import { clearActivity, formatDuration } from "../lib/api";
  import type { StatusPayload } from "../lib/types";
  import PageHeader from "../components/PageHeader.svelte";

  let {
    status,
    onMutate,
  }: { status: StatusPayload; onMutate: () => Promise<void> } = $props();
</script>

<div class="flex h-full flex-col gap-6 overflow-y-auto p-7">
  <div class="flex items-end justify-between">
    <PageHeader
      icon="list_bullets"
      title="Activity"
      subtitle="Timeline perubahan config, pairing, dan event Host."
    />
    <button
      class="neo-chip px-3 py-1.5 text-[12px] font-medium text-coral hover:text-tprimary"
      disabled={status.activity.length === 0}
      onclick={() => clearActivity().then(onMutate)}
    >
      Clear activity
    </button>
  </div>

  <div class="grid grid-cols-3 gap-4">
    <div class="neo-raised flex items-center gap-4 p-5">
      <span class="icon flex h-10 w-10 items-center justify-center rounded-xl bg-accent/10 text-[20px] text-accent-soft">{ICON.list_bullets}</span>
      <div>
        <div class="stat-value text-tprimary">{status.activity.length}</div>
        <div class="card-caption">Event tersimpan</div>
      </div>
    </div>
    <div class="neo-raised flex items-center gap-4 p-5">
      <span class="icon flex h-10 w-10 items-center justify-center rounded-xl bg-success/10 text-[20px] text-success">{ICON.plugs}</span>
      <div>
        <div class="stat-value text-success">{status.connectionCount}</div>
        <div class="card-caption">Device online</div>
      </div>
    </div>
    <div class="neo-raised flex items-center gap-4 p-5">
      <span class="icon flex h-10 w-10 items-center justify-center rounded-xl bg-amber/10 text-[20px] text-amber">{ICON.lightning}</span>
      <div>
        <div class="stat-value text-amber">{formatDuration(status.uptimeSecs)}</div>
        <div class="card-caption">Host uptime</div>
      </div>
    </div>
  </div>

  <div class="flex flex-col gap-2.5">
    {#if status.activity.length === 0}
      <div class="neo-raised p-5 text-[13px] text-tmuted">Belum ada aktivitas.</div>
    {:else}
      {#each [...status.activity].reverse() as event, i (i)}
        <div class="neo-raised flex items-center gap-3 px-4 py-2.5">
          <span class="w-7 shrink-0 font-mono text-[11px] text-tmuted">{String(i + 1).padStart(2, "0")}</span>
          <span class="icon text-[12px] text-accent-soft">{ICON.activity}</span>
          <span class="min-w-0 truncate text-[13px] text-tsecondary">{event}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>
