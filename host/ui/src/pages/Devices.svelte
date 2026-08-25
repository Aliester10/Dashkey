<script lang="ts">
  import { onMount } from "svelte";
  import { ICON } from "../lib/icons";
  import { clientSessions, devicesList, formatDuration, getStatus, revokeDevice } from "../lib/api";
  import { getConfirmCtx } from "../lib/confirm.svelte";
  import type { DeviceView, SessionView } from "../lib/types";
  import PageHeader from "../components/PageHeader.svelte";

  const confirm = getConfirmCtx();

  let devices = $state<DeviceView[]>([]);
  let sessions = $state<SessionView[]>([]);
  let online = $state(0);
  let error = $state<string | null>(null);

  async function refresh() {
    try {
      const [d, s, st] = await Promise.all([devicesList(), clientSessions(), getStatus()]);
      devices = d;
      sessions = s;
      online = st.connectionCount;
    } catch (e) {
      error = String(e);
    }
  }

  async function doRevoke(dev: DeviceView) {
    const ok = await confirm.requestConfirm({
      title: "Cabut akses?",
      message: `Device "${dev.device_name}" (${dev.device_id}) harus pair ulang untuk terhubung lagi.`,
      confirmLabel: "Cabut akses",
      danger: true,
    });
    if (!ok) return;
    try {
      await revokeDevice(dev.device_id);
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  onMount(() => {
    refresh();
    const t = setInterval(refresh, 2000);
    return () => clearInterval(t);
  });
</script>

<div class="flex h-full flex-col gap-6 overflow-y-auto p-7">
  <PageHeader icon="devices" title="Devices" subtitle="Pantau HP yang aktif dan kelola akses pairing." />

  {#if error}
    <div class="neo-raised p-4 text-[13px] text-coral">{error}</div>
  {/if}

  <div class="grid grid-cols-3 gap-4">
    <div class="neo-raised flex items-center gap-4 p-5">
      <span class="icon flex h-10 w-10 items-center justify-center rounded-xl bg-success/10 text-[20px] text-success">{ICON.plugs}</span>
      <div>
        <div class="stat-value text-success">{sessions.filter((s) => s.device_id).length}</div>
        <div class="card-caption">Sesi aktif</div>
      </div>
    </div>
    <div class="neo-raised flex items-center gap-4 p-5">
      <span class="icon flex h-10 w-10 items-center justify-center rounded-xl bg-accent/10 text-[20px] text-accent-soft">{ICON.devices}</span>
      <div>
        <div class="stat-value text-tprimary">{devices.length}</div>
        <div class="card-caption">Device ter-pairing</div>
      </div>
    </div>
    <div class="neo-raised flex items-center gap-4 p-5">
      <span class="icon flex h-10 w-10 items-center justify-center rounded-xl bg-purple/10 text-[20px] text-purple">{ICON.qr_code}</span>
      <div>
        <div class="stat-value text-purple">2 min</div>
        <div class="card-caption">Token pairing</div>
      </div>
    </div>
  </div>

  <section>
    <h2 class="mb-3 text-[12px] font-semibold tracking-wider text-tmuted">LIVE SESSIONS</h2>
    {#if sessions.length === 0}
      <div class="neo-raised p-5 text-[13px] text-tmuted">
        Belum ada HP yang terhubung. Buka halaman Pairing untuk menghubungkan device baru.
      </div>
    {:else}
      <div class="flex flex-col gap-3">
        {#each sessions as s (s.id)}
          <div class="neo-raised flex items-center gap-4 p-4">
            <span class="icon flex h-9 w-9 items-center justify-center rounded-xl bg-success/10 text-[18px] text-success">{ICON.plugs}</span>
            <div class="min-w-0">
              <div class="truncate text-[15px] font-semibold text-tprimary">
                {s.device_id ? devices.find((d) => d.device_id === s.device_id)?.device_name ?? s.device_id : "Menunggu autentikasi"}
              </div>
              <div class="truncate text-[11px] text-tmuted">
                {s.peer_ip}{s.device_id ? ` · ${s.device_id}` : ""}
              </div>
            </div>
            <div class="ml-auto">
              <span class="neo-inset px-3 py-1 text-[11px] font-medium text-success">
                {formatDuration(s.connected_secs)}
              </span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <section>
    <h2 class="mb-3 text-[12px] font-semibold tracking-wider text-tmuted">PAIRED DEVICES</h2>
    {#if devices.length === 0}
      <div class="neo-raised p-5 text-[13px] text-tmuted">Belum ada device ter-pairing.</div>
    {:else}
      <div class="flex flex-col gap-3">
        {#each devices as dev (dev.device_id)}
          <div class="neo-raised flex items-center gap-4 p-4">
            <span
              class={`icon flex h-8 w-8 items-center justify-center rounded-xl text-[16px] ${dev.online ? "bg-success/10 text-success" : "bg-surface-3/40 text-tmuted"}`}
            >{ICON.user_circle}</span>
            <div class="min-w-0 flex-1">
              <div class="truncate text-[14px] font-semibold text-tprimary">{dev.device_name}</div>
              <div class="truncate text-[11px] text-tmuted">{dev.device_id}</div>
            </div>
            <span
              class="neo-inset px-3 py-1 text-[11px] font-semibold"
              class:text-success={dev.online}
              class:text-tmuted={!dev.online}
            >
              {dev.online ? "ONLINE" : "OFFLINE"}
            </span>
            <button
              class="neo-chip px-3 py-1.5 text-[12px] font-medium text-coral hover:text-tprimary"
              onclick={() => doRevoke(dev)}
            >
              Cabut akses
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</div>
