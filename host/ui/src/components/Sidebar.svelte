<script lang="ts">
  import { ICON, type IconName } from "../lib/icons";

  export interface NavItem {
    id: string;
    label: string;
    icon: IconName;
  }

  let {
    items,
    active,
    online,
    onselect,
  }: {
    items: NavItem[];
    active: string;
    online: boolean;
    onselect: (id: string) => void;
  } = $props();
</script>

<aside
  class="flex h-full w-[228px] shrink-0 flex-col border-r border-white/5 bg-surface-1/60 px-4 py-5"
>
  <div class="flex items-center gap-3 px-2">
    <div
      class="neo-inset flex h-10 w-10 items-center justify-center text-[20px] text-amber icon"
    >
      {ICON.lightning}
    </div>
    <div class="leading-tight">
      <div class="text-[16px] font-bold tracking-tight text-tprimary">DashKey</div>
      <div class="text-[11px] font-medium text-tmuted">Host Controller</div>
    </div>
  </div>

  <div class="mt-6 flex flex-col gap-1">
    {#each items as item (item.id)}
      <button
        class="tab-pill flex items-center gap-3 text-left"
        class:tab-pill-active={active === item.id}
        onclick={() => onselect(item.id)}
      >
        <span class="icon text-[18px] leading-none">{ICON[item.icon]}</span>
        <span>{item.label}</span>
      </button>
    {/each}
  </div>

  <div class="mt-auto">
    <div class="divider mb-4"></div>
    <div class="flex items-center gap-2 px-2">
      <span
        class="inline-block h-2 w-2 rounded-full"
        class:bg-success={online}
        class:bg-tmuted={!online}
      ></span>
      <span class="text-[12px] text-tmuted">
        {online ? "Server aktif" : "Server idle"}
      </span>
    </div>
    <div class="mt-1 px-2 text-[11px] text-tmuted/70">DashKey Host v0.1.0</div>
  </div>
</aside>
