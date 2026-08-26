<script lang="ts">
  import { ICON } from "../lib/icons";

  let {
    title,
    onclose,
    children,
    width = 480,
  }: {
    title: string;
    onclose: () => void;
    children?: import("svelte").Snippet;
    width?: number;
  } = $props();
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6">
  <button class="absolute inset-0 cursor-default" aria-label="Tutup dialog" onclick={onclose}></button>
  <div
    class="neo-raised relative flex max-h-full w-full flex-col overflow-hidden"
    style={`max-width: ${width}px`}
  >
    <div class="flex items-center justify-between border-b border-border px-5 py-3.5">
      <h3 class="text-[15px] font-semibold text-tprimary">{title}</h3>
      <button class="neo-chip flex h-8 w-8 items-center justify-center text-tmuted icon hover:text-tprimary" onclick={onclose}>
        {ICON.x}
      </button>
    </div>
    <div class="min-h-0 flex-1 overflow-y-auto p-5">
      {#if children}
        {@render children()}
      {/if}
    </div>
  </div>
</div>
