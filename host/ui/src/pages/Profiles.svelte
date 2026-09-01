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

  let profileModal = $state<{ profileId: string | null; name: string } | null>(null);
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

  function openNewProfile() {
    profileModal = { profileId: null, name: "" };
  }

  function openRenameProfile(profileId: string, name: string) {
    profileModal = { profileId, name };
  }

  async function doNewPage(profileId: string) {
    await mutate(() => createPage(profileId));
  }

  async function doActivate(profileId: string) {
    await mutate(() => setActiveProfile(profileId));
  }

  async function doDeleteProfile(profileId: string) {
    const prof = config.profiles.find((p) => p.profile_id === profileId);
    const ok = await confirm.requestConfirm({
      title: "Hapus profile?",
      message: `Profile "${prof?.name ?? profileId}" akan dihapus beserta page yang tidak dipakai.`,
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

  async function saveProfile() {
    if (!profileModal) return;
    const m = profileModal;
    const name = m.name.trim();
    profileModal = null;
    if (!name) return;
    if (m.profileId) {
      await mutate(() => renameProfile(m.profileId!, name));
    } else {
      await mutate(() => createProfile(name));
    }
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

<div class="flex h-full flex-col gap-8 overflow-y-auto p-8">
  <div class="flex items-end justify-between">
    <PageHeader
      icon="user_circle"
      title="Profiles & Pages"
      subtitle="Workspace terpisah untuk streaming, gaming, kerja, dan kebutuhan lain."
    />
    <button class="btn-primary flex items-center gap-2 px-5 py-2.5" onclick={openNewProfile}>
      <span class="icon text-[15px]">{ICON.plus}</span>
      <span>Profile baru</span>
    </button>
  </div>

  <div class="flex flex-col gap-6">
    {#each config.profiles as profile (profile.profile_id)}
      {@const active = profile.profile_id === config.active_profile}
      <section
        class={`relative rounded-2xl border p-7 transition-colors duration-200 ${active ? "border-accent/25 bg-accent/[0.03]" : "border-border bg-surface-1 hover:border-tmuted/30"}`}
      >
        {#if active}
          <span class="absolute left-0 top-7 bottom-7 w-[3px] rounded-r-full bg-accent"></span>
        {/if}

        <div class="flex items-center gap-4">
          <span
            class={`icon flex h-11 w-11 shrink-0 items-center justify-center rounded-full text-[19px] ${active ? "bg-accent/10 text-accent" : "bg-surface-3/30 text-tsecondary"}`}
          >{ICON.user_circle}</span>
          <div class="min-w-0">
            <div class="flex items-center gap-2.5">
              <h3 class="truncate text-[16px] font-semibold tracking-tight text-tprimary">{profile.name}</h3>
              {#if active}
                <span class="rounded-full bg-accent/10 px-2.5 py-0.5 text-[9.5px] font-semibold tracking-[0.14em] text-accent">AKTIF</span>
              {/if}
            </div>
            <p class="mt-0.5 text-[12px] text-tmuted">{profile.pages.length} page · {profile.profile_id}</p>
          </div>
          <div class="ml-auto flex shrink-0 items-center gap-1">
            {#if !active}
              <button
                class="rounded-lg px-3.5 py-2 text-[12.5px] font-medium text-tsecondary transition-colors hover:bg-hover hover:text-tprimary"
                onclick={() => doActivate(profile.profile_id)}
              >
                Aktifkan
              </button>
            {/if}
            <button
              class="flex items-center gap-1.5 rounded-lg px-3.5 py-2 text-[12.5px] font-medium text-tsecondary transition-colors hover:bg-hover hover:text-tprimary"
              onclick={() => openRenameProfile(profile.profile_id, profile.name)}
            >
              <span class="icon text-[13px]">{ICON.pencil}</span>
              Rename
            </button>
            <button
              class="flex items-center gap-1.5 rounded-lg px-3.5 py-2 text-[12.5px] font-medium text-tmuted transition-colors hover:bg-coral/10 hover:text-coral"
              onclick={() => doDeleteProfile(profile.profile_id)}
            >
              <span class="icon text-[13px]">{ICON.trash}</span>
              Hapus
            </button>
          </div>
        </div>

        <div class="mt-6">
          <p class="mb-3 text-[10.5px] font-semibold tracking-[0.14em] text-tmuted">PAGES</p>
          <div class="grid grid-cols-[repeat(auto-fill,minmax(210px,1fr))] gap-3">
            {#each profile.pages as pageId (pageId)}
              {@const page = config.pages[pageId]}
              {#if page}
                {@const pageActive = pageId === config.active_page}
                {@const btnCount = page.buttons.filter((b) => b !== null).length}
                <div
                  class={`group relative flex flex-col rounded-xl border p-4 transition-all duration-200 ${pageActive ? "border-accent/30 bg-accent/[0.05]" : "border-border bg-surface-2 hover:-translate-y-0.5 hover:border-tmuted/40 hover:bg-surface-3/20"}`}
                >
                  <div class="flex items-start justify-between">
                    <span
                      class={`icon flex h-8 w-8 items-center justify-center rounded-lg text-[15px] ${pageActive ? "bg-accent/10 text-accent" : "bg-surface-3/40 text-tsecondary"}`}
                    >{ICON.stack}</span>
                    <div class="flex gap-0.5 opacity-0 transition-opacity duration-150 group-hover:opacity-100">
                      <button
                        class="rounded-md p-1.5 text-tmuted transition-colors hover:bg-hover hover:text-tprimary"
                        title="Edit page"
                        aria-label={`Edit page ${page.name}`}
                        onclick={() => (editPage = { pageId, name: page.name, rows: page.grid_size.rows, cols: page.grid_size.cols, pageType: page.page_type })}
                      >
                        <span class="icon text-[13px]">{ICON.pencil}</span>
                      </button>
                      <button
                        class="rounded-md p-1.5 text-tmuted transition-colors hover:bg-coral/10 hover:text-coral"
                        title="Hapus page"
                        aria-label={`Hapus page ${page.name}`}
                        onclick={() => doDeletePage(pageId, page.name)}
                      >
                        <span class="icon text-[13px]">{ICON.trash}</span>
                      </button>
                    </div>
                  </div>
                  <div class="mt-3.5 min-w-0">
                    <div class="truncate text-[13px] font-semibold tracking-tight text-tprimary">{page.name}</div>
                    <div class="mt-1 text-[11px] text-tmuted">
                      {page.grid_size.rows}×{page.grid_size.cols} · {btnCount} tombol
                      {#if page.page_type === "trackpad"}· trackpad{/if}
                    </div>
                  </div>
                </div>
              {/if}
            {/each}
            <button
              class="flex min-h-[104px] flex-col items-center justify-center gap-1.5 rounded-xl border border-dashed border-border text-tmuted transition-colors hover:border-accent/40 hover:text-accent"
              onclick={() => doNewPage(profile.profile_id)}
            >
              <span class="icon text-[16px]">{ICON.plus}</span>
              <span class="text-[11.5px] font-medium">Page baru</span>
            </button>
          </div>
        </div>
      </section>
    {/each}
  </div>
</div>

{#if profileModal}
  <Modal
    title={profileModal.profileId ? "Rename Profile" : "Profile Baru"}
    onclose={() => (profileModal = null)}
  >
    <label class="card-caption block mb-1.5" for="profile-name">Nama profile</label>
    <input
      id="profile-name"
      bind:value={profileModal.name}
      class="field-input w-full"
      placeholder="mis. Streaming, Gaming, Kerja…"
      onkeydown={(e) => e.key === "Enter" && saveProfile()}
    />
    <div class="mt-5 flex justify-end gap-3">
      <button class="rounded-lg px-4 py-2 text-[13px] font-medium text-tsecondary transition-colors hover:bg-hover hover:text-tprimary" onclick={() => (profileModal = null)}>Batal</button>
      <button class="btn-primary px-5 py-2" onclick={saveProfile}>Simpan</button>
    </div>
  </Modal>
{/if}

{#if editPage}
  <Modal title="Edit Page" onclose={() => (editPage = null)}>
    <div class="flex flex-col gap-3">
      <div>
        <label class="card-caption block mb-1.5" for="page-name">Nama page</label>
        <input id="page-name" bind:value={editPage.name} class="field-input w-full" />
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
        <select id="page-type" class="field-input w-full" bind:value={editPage.pageType}>
          <option value="buttons">Grid tombol</option>
          <option value="trackpad">Trackpad</option>
        </select>
      </div>
    </div>
    <div class="mt-5 flex justify-end gap-3">
      <button class="rounded-lg px-4 py-2 text-[13px] font-medium text-tsecondary transition-colors hover:bg-hover hover:text-tprimary" onclick={() => (editPage = null)}>Batal</button>
      <button class="btn-primary px-5 py-2" onclick={savePage}>Simpan</button>
    </div>
  </Modal>
{/if}
