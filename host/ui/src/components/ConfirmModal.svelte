<script lang="ts">
  let { ctx }: { ctx: import("../lib/confirm.svelte.ts").ConfirmCtx } = $props();
  const p = $derived(ctx.state.pending);
</script>

{#if p}
  <div class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 p-6">
    <div class="neo-raised w-full max-w-[420px] p-6">
      <h3 class="text-[16px] font-semibold text-tprimary">{p.req.title}</h3>
      <p class="mt-2 text-[13px] leading-relaxed text-tsecondary">{p.req.message}</p>
      <div class="mt-5 flex justify-end gap-3">
        <button
          class="neo-chip px-4 py-2 text-[13px] font-medium text-tsecondary hover:text-tprimary"
          onclick={() => ctx.settle(false)}
        >
          Batal
        </button>
        <button
          class={`rounded-xl px-4 py-2 text-[13px] font-semibold ${p.req.danger ? "bg-coral/90 text-white" : "bg-accent/90 text-btn-text"}`}
          onclick={() => ctx.settle(true)}
        >
          {p.req.confirmLabel ?? "Ya, lanjutkan"}
        </button>
      </div>
    </div>
  </div>
{/if}
