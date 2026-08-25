<script lang="ts">
  import { ICON, type IconName } from "../lib/icons";

  let { icon, title, subtitle }: { icon: IconName; title: string; subtitle: string } = $props();
</script>

<div class="flex items-center gap-4">
  <div class="neo-inset flex h-14 w-14 shrink-0 items-center justify-center text-[26px] text-accent-soft icon">
    {ICON[icon]}
  </div>
  <div>
    <h1 class="text-[20px] font-bold tracking-tight text-tprimary">{title}</h1>
    <p class="mt-0.5 text-[13px] text-tsecondary">{subtitle}</p>
  </div>
</div>
