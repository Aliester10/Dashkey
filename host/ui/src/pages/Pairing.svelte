<script lang="ts">
  import { onMount } from "svelte";
  import { ICON } from "../lib/icons";
  import { pairGenerate } from "../lib/api";
  import type { PairPayload } from "../lib/types";
  import { formatDuration } from "../lib/api";
  import PageHeader from "../components/PageHeader.svelte";

  let pair = $state<PairPayload | null>(null);
  let generatedAt = $state<number | null>(null);
  let now = $state(Date.now());
  let copying = $state(false);

  const remaining = $derived(
    pair && generatedAt !== null ? Math.max(0, pair.ttlSecs - (now - generatedAt) / 1000) : 0,
  );
  const expired = $derived(remaining <= 0);

  async function generate() {
    pair = await pairGenerate();
    generatedAt = Date.now();
    now = Date.now();
  }

  async function copyPayload() {
    if (!pair) return;
    try {
      await navigator.clipboard.writeText(pair.payload);
      copying = true;
      setTimeout(() => (copying = false), 1200);
    } catch {
      /* clipboard tidak tersedia */
    }
  }

  onMount(() => {
    const t = setInterval(() => (now = Date.now()), 500);
    return () => clearInterval(t);
  });
</script>

<div class="flex h-full flex-col gap-6 overflow-y-auto p-7">
  <PageHeader
    icon="qr_code"
    title="Pairing HP Baru"
    subtitle="Hubungkan smartphone sebagai controller DashKey."
  />

  <div class="flex gap-6">
    <div class="neo-raised flex flex-1 flex-col items-center p-8">
      <div class="flex items-center gap-3 self-start">
        <button class="btn-primary flex items-center gap-2 px-4 py-2" onclick={generate}>
          <span class="icon text-[16px]">{ICON.qr_code}</span>
          <span>Generate QR Baru</span>
        </button>
        {#if pair}
          <span
            class="neo-inset px-3 py-1.5 text-[12px] font-medium"
            class:text-coral={expired}
            class:text-success={!expired}
          >
            {expired ? "QR kedaluwarsa — generate ulang" : `Berlaku ${formatDuration(Math.ceil(remaining))} lagi`}
          </span>
        {/if}
      </div>

      <div class="mt-8">
        {#if pair && !expired}
          <div class="neo-inset p-4">
            <img
              src={`data:image/svg+xml;utf8,${encodeURIComponent(pair.qrSvg)}`}
              alt="QR pairing DashKey"
              class="h-[260px] w-[260px] rounded-lg"
            />
          </div>
        {:else if pair}
          <div class="neo-inset flex h-[260px] w-[260px] items-center justify-center text-[13px] text-coral">
            QR sudah tidak berlaku
          </div>
        {:else}
          <div class="neo-inset flex h-[260px] w-[260px] items-center justify-center text-center text-[13px] text-tmuted">
            Belum ada QR.<br />Klik "Generate QR Baru" untuk mulai pairing.
          </div>
        {/if}
      </div>

      {#if pair}
        <div class="mt-6 w-full max-w-[460px]">
          <label class="card-caption block mb-1.5" for="pair-payload">Payload</label>
          <div class="flex gap-2">
            <input
              id="pair-payload"
              readonly
              value={pair.payload}
              class="neo-inset min-w-0 flex-1 px-3 py-2 font-mono text-[12px] text-tsecondary outline-none"
            />
            <button class="neo-chip px-3 py-2 text-[12px] font-medium text-tsecondary hover:text-tprimary" onclick={copyPayload}>
              {copying ? "Tersalin ✓" : "Salin"}
            </button>
          </div>
        </div>
      {/if}
    </div>

    <div class="neo-raised flex w-[300px] flex-col p-6">
      <h3 class="card-title">Langkah pairing</h3>
      <ol class="mt-4 flex flex-col gap-4 text-[13px] leading-relaxed text-tsecondary">
        <li class="flex gap-3">
          <span class="neo-inset flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-[13px] font-bold text-accent-soft">1</span>
          Buka aplikasi DashKey di HP Anda.
        </li>
        <li class="flex gap-3">
          <span class="neo-inset flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-[13px] font-bold text-accent-soft">2</span>
          Scan QR di samping (atau ketik payload manual).
        </li>
        <li class="flex gap-3">
          <span class="neo-inset flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-[13px] font-bold text-accent-soft">3</span>
          HP otomatis terhubung — cek statusnya di tab Devices.
        </li>
      </ol>
      <div class="divider my-5"></div>
      <p class="text-[12px] leading-relaxed text-tmuted">
        Token berlaku <span class="font-semibold text-tsecondary">2 menit</span>. Setelah
        ter-pairing, koneksi otomatis kembali tanpa QR (token permanen tersimpan di HP).
      </p>
    </div>
  </div>
</div>
