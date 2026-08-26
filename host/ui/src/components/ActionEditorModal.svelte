<script lang="ts">
  import Modal from "./Modal.svelte";
  import { setButtonActions } from "../lib/api";
  import { ACTION_TYPES, describeAction } from "../lib/constants";
  import type { Button } from "../lib/types";

  let {
    button,
    onMutate,
    onclose,
  }: {
    button: Button;
    onMutate: () => Promise<void>;
    onclose: () => void;
  } = $props();

  let editor = $state<{ draftType: string; text: string; media: string; editing: number | null } | null>(
    { draftType: ACTION_TYPES[0].key, text: "", media: "play_pause", editing: null },
  );

  const editorType = $derived(
    ACTION_TYPES.find((t) => t.key === editor?.draftType) ?? ACTION_TYPES[0],
  );

  function buildAction(draftType: string, text: string, media: string): Record<string, unknown> {
    const t = text.trim();
    switch (draftType) {
      case "open_app":
        return { action_type: "open_app", target: t };
      case "close_app":
        return { action_type: "close_app", target: t, force: media === "force" };
      case "open_url":
        return { action_type: "open_url", target: t };
      case "shell":
        return { action_type: "shell", command: t };
      case "hotkey":
        return {
          action_type: "hotkey",
          keys: t
            .split(",")
            .map((s) => s.trim().toLowerCase())
            .filter(Boolean),
        };
      case "play_sound":
        return { action_type: "play_sound", target: t };
      case "media_control":
        return { action_type: "media_control", control: media };
      case "obs_switch_scene":
        return { action_type: "obs_switch_scene", target: t };
      case "obs_toggle_mute":
        return { action_type: "obs_toggle_mute", target: t };
      case "obs_start_stream":
        return { action_type: "obs_start_stream" };
      case "obs_stop_stream":
        return { action_type: "obs_stop_stream" };
      case "obs_start_recording":
        return { action_type: "obs_start_recording" };
      case "obs_stop_recording":
        return { action_type: "obs_stop_recording" };
      default:
        return { action_type: "open_app", target: t };
    }
  }

  function editAction(index: number) {
    const a = button.actions[index] as Record<string, unknown>;
    const draftType = String(a.action_type ?? ACTION_TYPES[0].key);
    const text = String(a.target ?? a.command ?? (Array.isArray(a.keys) ? a.keys.join(",") : "") ?? "");
    const media = String(a.control ?? (a.force ? "force" : ""));
    editor = { draftType, text, media, editing: index };
  }

  async function saveActions(actions: unknown[]) {
    await setButtonActions(button.button_id, actions);
    await onMutate();
  }

  async function saveForm() {
    if (!editor) return;
    const ed = editor;
    const actions: unknown[] = [...button.actions];
    const action = buildAction(ed.draftType, ed.text, ed.media);
    if (ed.editing !== null && ed.editing < actions.length) {
      actions[ed.editing] = action;
    } else {
      actions.push(action);
    }
    await saveActions(actions);
    editor = { draftType: ed.draftType, text: "", media: ed.media, editing: null };
  }

  async function op(index: number, kind: "del" | "up" | "down") {
    const actions: unknown[] = [...button.actions];
    if (kind === "del") actions.splice(index, 1);
    if (kind === "up" && index > 0) [actions[index - 1], actions[index]] = [actions[index], actions[index - 1]];
    if (kind === "down" && index + 1 < actions.length) [actions[index], actions[index + 1]] = [actions[index + 1], actions[index]];
    await saveActions(actions);
  }
</script>

<Modal title={`Action Editor — ${button.label}`} width={520} onclose={onclose}>
  <div class="text-[11px] text-tmuted">{button.button_id}</div>
  <div class="divider my-3"></div>

  <!-- Daftar aksi -->
  <div class="flex flex-col gap-1.5">
    {#if button.actions.length === 0}
      <p class="rounded-lg bg-surface-0/60 px-3 py-2 text-[12.5px] text-tmuted">
        Belum ada aksi. Tambahkan di bawah.
      </p>
    {:else}
      {#each button.actions as action, i (i)}
        <div class="flex items-center gap-2 rounded-lg bg-surface-0/60 px-3 py-2">
          <span class="text-[11px] font-mono text-tmuted">{i + 1}</span>
          <span class="min-w-0 flex-1 truncate text-[12.5px] text-tsecondary">{describeAction(action)}</span>
          <div class="flex shrink-0 items-center gap-1">
            <button class="rounded px-1.5 text-[11px] text-tmuted hover:text-tprimary" onclick={() => editAction(i)}>Edit</button>
            <button class="rounded px-1 text-[11px] text-tmuted hover:text-tprimary" disabled={i === 0} onclick={() => op(i, "up")}>↑</button>
            <button class="rounded px-1 text-[11px] text-tmuted hover:text-tprimary" disabled={i === button.actions.length - 1} onclick={() => op(i, "down")}>↓</button>
            <button class="rounded px-1.5 text-[11px] text-coral hover:text-tprimary" onclick={() => op(i, "del")}>Del</button>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <div class="divider my-3"></div>

  <!-- Form aksi -->
  {#if editor}
    {@const ed = editor}
    <div class="text-[13px] font-semibold text-tprimary">
      {ed.editing !== null ? "Edit action" : "Add new action"}
    </div>
    <div class="mt-3 flex flex-col gap-3">
      <div>
        <label class="card-caption block mb-1.5" for="ae-type">TYPE</label>
        <select
          id="ae-type"
          class="neo-inset w-full px-3 py-2 text-[13px] text-tprimary outline-none"
          value={ed.draftType}
          onchange={(e) => (ed.draftType = e.currentTarget.value)}
        >
          {#each ACTION_TYPES as t (t.key)}
            <option value={t.key}>{t.label}</option>
          {/each}
        </select>
      </div>

      {#if editorType.key === "media_control"}
        <div>
          <label class="card-caption block mb-1.5" for="ae-control">CONTROL</label>
          <select id="ae-control" class="neo-inset w-full px-3 py-2 text-[13px] text-tprimary outline-none" bind:value={ed.media}>
            {#each ["play_pause", "next", "prev", "stop", "volume_up", "volume_down", "mute"] as c (c)}
              <option value={c}>{c}</option>
            {/each}
          </select>
        </div>
      {:else if ["obs_start_stream", "obs_stop_stream", "obs_start_recording", "obs_stop_recording"].includes(editorType.key)}
        <p class="text-[12.5px] text-tsecondary">Aksi ini tidak memerlukan parameter.</p>
      {:else}
        <div>
          <label class="card-caption block mb-1.5" for="ae-target">
            {editorType.key === "hotkey" ? "KEYS (comma-separated)" : editorType.key === "close_app" ? "PROSES" : "TARGET"}
          </label>
          <input
            id="ae-target"
            bind:value={ed.text}
            class="neo-inset w-full px-3 py-2 text-[13px] text-tprimary outline-none"
            placeholder={editorType.hint || "…"}
            onkeydown={(e) => e.key === "Enter" && saveForm()}
          />
        </div>
      {/if}

      {#if editorType.key === "close_app"}
        <label class="flex items-center gap-2 text-[12.5px] text-tsecondary">
          <input type="checkbox" class="accent-accent" checked={ed.media === "force"} onchange={(e) => (ed.media = e.currentTarget.checked ? "force" : "")} />
          Force close (paksa, tanpa simpan data)
        </label>
      {/if}
    </div>

    <div class="mt-5 flex justify-end gap-3">
      <button class="neo-chip px-4 py-2 text-[13px] font-medium text-tsecondary hover:text-tprimary" onclick={onclose}>
        Tutup
      </button>
      <button class="btn-primary px-4 py-2" onclick={saveForm}>Save Action</button>
    </div>
  {/if}
</Modal>