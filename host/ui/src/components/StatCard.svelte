<script lang="ts">
  import { ICON, type IconName } from "../lib/icons";

  type Accent = "success" | "purple" | "accent" | "muted";

  const accentText: Record<Accent, string> = {
    success: "text-success",
    purple: "text-purple",
    accent: "text-accent-soft",
    muted: "text-tsecondary",
  };
  const accentBg: Record<Accent, string> = {
    success: "bg-success/10",
    purple: "bg-purple/10",
    accent: "bg-accent/10",
    muted: "bg-surface-3/40",
  };

  let {
    icon,
    label,
    value,
    caption,
    accent = "accent",
    onclick,
  }: {
    icon: IconName;
    label: string;
    value: string;
    caption: string;
    accent?: Accent;
    onclick?: () => void;
  } = $props();
</script>

<button
  class="neo-chip flex w-full cursor-pointer flex-col gap-1.5 px-5 py-4 text-left"
  onclick={onclick}
>
  <div class="flex items-center justify-between">
    <span class="text-[11px] font-semibold tracking-wider text-tmuted">{label}</span>
    <span class={`icon flex h-7 w-7 items-center justify-center rounded-lg text-[15px] ${accentBg[accent]} ${accentText[accent]}`}>
      {ICON[icon]}
    </span>
  </div>
  <div class={`stat-value ${accentText[accent]}`}>{value}</div>
  <div class="text-[11.5px] text-tmuted">{caption}</div>
</button>
