<script lang="ts">
  import { slide } from "svelte/transition";
  import { ICON, type IconName } from "../lib/icons";

  let {
    title,
    icon,
    count,
    open = $bindable(),
    children,
  }: {
    title: string;
    icon: IconName;
    count?: number;
    open: boolean;
    children?: import("svelte").Snippet;
  } = $props();
</script>

<div class="border-b border-border">
  <button
    class="flex w-full items-center gap-2.5 px-4 py-2.5 text-left text-[12.5px] font-semibold text-tprimary hover:bg-hover"
    onclick={() => (open = !open)}
    aria-expanded={open}
  >
    <span class="icon text-[15px] text-accent-soft">{ICON[icon]}</span>
    <span class="min-w-0 flex-1 truncate">{title}</span>
    {#if count !== undefined}
      <span class="rounded-md bg-surface-3/50 px-1.5 py-0.5 text-[10.5px] font-medium text-tmuted">{count}</span>
    {/if}
    <span class={`text-[10px] text-tmuted transition-transform duration-150 ${open ? "rotate-90" : ""}`}>▸</span>
  </button>
  {#if open}
    <div class="px-3 pb-3" transition:slide={{ duration: 160 }}>
      {#if children}
        {@render children()}
      {/if}
    </div>
  {/if}
</div>