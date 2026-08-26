<script lang="ts">
  import { ICON } from "../lib/icons";
  import {
    createPage,
    createProfile,
    deletePage,
    deleteProfile,
    renameProfile,
    setActiveProfile,
    updatePage,
  } from "../lib/api";
  import { getConfirmCtx } from "../lib/confirm.svelte";
  import type { Config } from "../lib/types";
  import PageHeader from "../components/PageHeader.svelte";
  import Modal from "../components/Modal.svelte";

  const confirm = getConfirmCtx();

  let {
    config,
    onMutate,
  }: { config: Config; onMutate: () => Promise<void> } = $props();

  let renameFor = $state<{ profileId: string; name: string } | null>(null);
  let editPage = $state<{ pageId: string; name: string; rows: number; cols: number; pageType: string } | null>(null);
  let busy = $state(false);

  async function mutate(fn: () => Promise<unknown>) {
    busy = true;
    try {
      await fn();
      await onMutate();
    } finally {
      busy = false;
    }
  }

  async function doNewProfile() {
    await mutate(() => createProfile());
  }

  async function doNewPage(profileId: string) {
    await mutate(() => createPage(profileId));
  }

  async function doActivate(profileId: string) {
    await mutate(() => setActiveProfile(profileId));
  }

  async function doDeleteProfile(profileId: string) {
    const ok = await confirm.requestConfirm({
      title: "Hapus profile?",
      message: `Profile "${profileId}" akan dihapus beserta page yang tidak dipakai.`,
      confirmLabel: "Hapus",
      danger: true,
    });
    if (ok) await mutate(() => deleteProfile(profileId));
  }

  async function doDeletePage(pageId: string, name: string) {
    const ok = await confirm.requestConfirm({
      title: "Hapus page?",
      message: `Page "${name}" (${pageId}) akan dihapus dari semua profile.`,
      confirmLabel: "Hapus",
      danger: true,
    });
    if (ok) await mutate(() => deletePage(pageId));
  }

  async function saveRename() {
    if (!renameFor) return;
    const r = renameFor;
    if (r.name.trim()) {
      await mutate(() => renameProfile(r.profileId, r.name.trim()));
    }
    renameFor = null;
  }

  async function savePage() {
    if (!editPage) return;
    const e = editPage;
    if (e.name.trim()) {
      await mutate(() => updatePage(e.pageId, e.name.trim(), e.rows, e.cols, e.pageType));
    }
    editPage = null;
  }
</script>

<div class="flex h-full flex-col gap-6 overflow-y-auto p-7">
  <div class="flex items-end justify-between">
    <PageHeader
      icon="user_circle"
      title="Profiles & Pages"
      subtitle="Workspace terpisah untuk streaming, gaming, kerja, dan kebutuhan lain."
    />
    <button class="btn-primary flex items-center gap-2 px-4 py-2" onclick={doNewProfile}>
      <span class="icon text-[15px]">{ICON.plus}</span>
      <span>Profile baru</span>
    </button>
  </div>

  <div class="flex flex-col gap-5">
    {#each config.profiles as profile (profile.profile_id)}
      {@const active = profile.profile_id === config.active_profile}
      <div
        class={`neo-raised p-5 ${active ? "outline outline-2 outline-purple/50" : ""}`}
      >
        <div class="flex items-center gap-3">
          <span
            class={`icon flex h-10 w-10 items-center justify-center rounded-xl text-[18px] ${active ? "bg-purple/15 text-purple" : "bg-surface-3/40 text-tsecondary"}`}
          >{ICON.user_circle}</span>
          <div>
            <div class="text-[16px] font-semibold text-tprimary">{profile.name}</div>
            <div class="text-[11px] text-tmuted">{profile.pages.length} page · {profile.profile_id}</div>
          </div>
          <div class="ml-auto flex items-center gap-2">
            {#if active}
              <span class="neo-inset px-3 py-1 text-[11px] font-semibold text-success">ACTIVE</span>
            {:else}
              <button class="neo-chip px-3 py-1.5 text-[12px] font-medium text-accent-soft hover:text-tprimary" onclick={() => doActivate(profile.profile_id)}>
                Aktifkan
              </button>
            {/if}
            <button class="neo-chip px-3 py-1.5 text-[12px] font-medium text-tsecondary hover:text-tprimary" onclick={() => (renameFor = { profileId: profile.profile_id, name: profile.name })}>
              Rename
            </button>
            <button class="neo-chip px-3 py-1.5 text-[12px] font-medium text-coral hover:text-tprimary" onclick={() => doDeleteProfile(profile.profile_id)}>
              Hapus
            </button>
          </div>
        </div>

        <div class="mt-4 flex flex-wrap gap-3">
          {#each profile.pages as pageId (pageId)}
            {@const page = config.pages[pageId]}
            {#if page}
              {@const pageActive = pageId === config.active_page}
              <div
                class={`neo-chip flex min-w-[180px] flex-col gap-1 px-4 py-3 ${pageActive ? "ring-1 ring-accent/40" : ""}`}
              >
                <div class="flex items-center gap-2">
                  <span class="icon text-[15px] text-purple">{ICON.stack}</span>
                  <span class="truncate text-[13px] font-semibold text-tprimary">{page.name}</span>
                </div>
                <div class="text-[11px] text-tmuted">
                  {page.grid_size.rows}×{page.grid_size.cols} · {page.buttons.length} tombol
                  {#if page.page_type === "trackpad"}· trackpad{/if}
                </div>
                <div class="mt-1 flex gap-1.5">
                  <button
                    class="rounded px-2 py-0.5 text-[11px] text-tsecondary hover:bg-hover"
                    onclick={() => (editPage = { pageId, name: page.name, rows: page.grid_size.rows, cols: page.grid_size.cols, pageType: page.page_type })}
                  >
                    Edit
                  </button>
                  <button class="rounded px-2 py-0.5 text-[11px] text-coral hover:bg-hover" onclick={() => doDeletePage(pageId, page.name)}>
                    Hapus
                  </button>
                </div>
              </div>
            {/if}
          {/each}
          <button
            class="neo-chip flex min-w-[180px] flex-col items-center justify-center gap-1 px-4 py-3 text-accent-soft hover:text-tprimary"
            onclick={() => doNewPage(profile.profile_id)}
          >
            <span class="icon text-[16px]">{ICON.plus}</span>
            <span class="text-[12px] font-medium">Page baru</span>
          </button>
        </div>
      </div>
    {/each}
  </div>
</div>

{#if renameFor}
  <Modal title="Rename Profile" onclose={() => (renameFor = null)}>
    <label class="card-caption block mb-1.5" for="profile-name">Nama profile</label>
    <input
      id="profile-name"
      bind:value={renameFor.name}
      class="neo-inset w-full px-3 py-2 text-[13px] text-tprimary outline-none"
      onkeydown={(e) => e.key === "Enter" && saveRename()}
    />
    <div class="mt-5 flex justify-end gap-3">
      <button class="neo-chip px-4 py-2 text-[13px] font-medium text-tsecondary hover:text-tprimary" onclick={() => (renameFor = null)}>Batal</button>
      <button class="btn-primary px-4 py-2" onclick={saveRename}>Simpan</button>
    </div>
  </Modal>
{/if}

{#if editPage}
  <Modal title="Edit Page" onclose={() => (editPage = null)}>
    <div class="flex flex-col gap-3">
      <div>
        <label class="card-caption block mb-1.5" for="page-name">Nama page</label>
        <input id="page-name" bind:value={editPage.name} class="neo-inset w-full px-3 py-2 text-[13px] text-tprimary outline-none" />
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="card-caption block mb-1.5" for="page-rows">Baris ({editPage.rows})</label>
          <input id="page-rows" type="range" min="1" max="8" bind:value={editPage.rows} class="w-full accent-accent" />
        </div>
        <div>
          <label class="card-caption block mb-1.5" for="page-cols">Kolom ({editPage.cols})</label>
          <input id="page-cols" type="range" min="1" max="8" bind:value={editPage.cols} class="w-full accent-accent" />
        </div>
      </div>
      <div>
        <label class="card-caption block mb-1.5" for="page-type">Tipe page</label>
        <select id="page-type" class="neo-inset w-full px-3 py-2 text-[13px] text-tprimary outline-none" bind:value={editPage.pageType}>
          <option value="buttons">Grid tombol</option>
          <option value="trackpad">Trackpad</option>
        </select>
      </div>
    </div>
    <div class="mt-5 flex justify-end gap-3">
      <button class="neo-chip px-4 py-2 text-[13px] font-medium text-tsecondary hover:text-tprimary" onclick={() => (editPage = null)}>Batal</button>
      <button class="btn-primary px-4 py-2" onclick={savePage}>Simpan</button>
    </div>
  </Modal>
{/if}
