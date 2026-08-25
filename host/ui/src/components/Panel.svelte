<script lang="ts">
  let { title, children }: { title: string; children?: import("svelte").Snippet } = $props();
</script>

<section class="neo-raised flex flex-col p-5">
  <h2 class="card-title">{title}</h2>
  <div class="mt-3 flex-1">
    {#if children}
      {@render children()}
    {/if}
  </div>
</section>
