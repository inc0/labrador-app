<script lang="ts">
  import { queue, view } from './stores'
  import type { QueuedImage } from './stores'
  import { v4 as uuidv4 } from 'uuid'

  let galleryInput: HTMLInputElement
  let dragging = false

  // ── viewfinder state ─────────────────────────────────────────────────────────
  let viewfinderOpen = false
  let videoEl: HTMLVideoElement
  let canvasEl: HTMLCanvasElement
  let stream: MediaStream | null = null
  let cameraError = ''

  async function openCamera() {
    cameraError = ''
    viewfinderOpen = true
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: 'environment', width: { ideal: 1920 }, height: { ideal: 1080 } },
        audio: false,
      })
      // wait for the video element to mount
      await new Promise(r => setTimeout(r, 50))
      videoEl.srcObject = stream
      await videoEl.play()
    } catch (e: any) {
      cameraError = e?.message ?? 'Camera unavailable'
    }
  }

  function closeCamera() {
    stream?.getTracks().forEach(t => t.stop())
    stream = null
    viewfinderOpen = false
    cameraError = ''
  }

  function capture() {
    if (!videoEl || !canvasEl) return
    canvasEl.width  = videoEl.videoWidth
    canvasEl.height = videoEl.videoHeight
    canvasEl.getContext('2d')!.drawImage(videoEl, 0, 0)
    canvasEl.toBlob(blob => {
      if (!blob) return
      const file = new File([blob], `photo-${Date.now()}.jpg`, { type: 'image/jpeg' })
      const dataUrl = URL.createObjectURL(blob)
      queue.update(q => [...q, { id: uuidv4(), dataUrl, data: '', mime: 'image/jpeg', file }])
      closeCamera()
    }, 'image/jpeg', 0.92)
  }

  // ── gallery / queue helpers ──────────────────────────────────────────────────
  function addFiles(files: FileList | File[]) {
    const existing = new Set($queue.map(q => q.file.name + q.file.size))
    const toAdd: QueuedImage[] = []
    for (const file of files) {
      if (!file.type.startsWith('image/')) continue
      if (existing.has(file.name + file.size)) continue
      const dataUrl = URL.createObjectURL(file)
      toAdd.push({ id: uuidv4(), dataUrl, data: '', mime: file.type, file })
    }
    if (toAdd.length) queue.update(q => [...q, ...toAdd])
    if (galleryInput) galleryInput.value = ''
  }

  function removeImage(id: string) {
    queue.update(q => q.filter(img => img.id !== id))
  }

  async function readBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => resolve((reader.result as string).split(',')[1])
      reader.onerror = reject
      reader.readAsDataURL(file)
    })
  }

  async function processQueue() {
    const withData = await Promise.all(
      $queue.map(async img => ({ ...img, data: await readBase64(img.file) }))
    )
    queue.set(withData)
    view.set('processing')
  }
</script>

<!-- ── Viewfinder overlay ──────────────────────────────────────────────────── -->
{#if viewfinderOpen}
  <div class="fixed inset-0 z-50 bg-black flex flex-col">
    {#if cameraError}
      <div class="flex-1 flex flex-col items-center justify-center gap-4 px-8">
        <p class="text-[#ff6b6b] text-center text-sm">{cameraError}</p>
        <button class="px-6 py-3 rounded-xl bg-[#2e3147] text-[#e2e4f0]" onclick={closeCamera}>Close</button>
      </div>
    {:else}
      <!-- Live feed -->
      <!-- svelte-ignore a11y_media_has_caption -->
      <video bind:this={videoEl} class="flex-1 w-full object-cover" playsinline autoplay muted></video>

      <!-- Controls -->
      <div class="flex items-center justify-between px-8 py-6 bg-black/60">
        <button class="text-white/70 text-sm active:opacity-60 w-16" onclick={closeCamera}>Cancel</button>
        <!-- Shutter -->
        <button
          class="w-16 h-16 rounded-full border-4 border-white bg-white/20 active:bg-white/50 transition-colors"
          onclick={capture}
          aria-label="Take photo"
        ></button>
        <div class="w-16"></div>
      </div>
    {/if}
  </div>
  <canvas bind:this={canvasEl} class="hidden"></canvas>
{/if}

<!-- ── Main view ──────────────────────────────────────────────────────────── -->
<div class="flex flex-col h-full">
  <header class="flex items-center justify-between px-4 py-3 border-b border-[#2e3147]">
    <h1 class="text-lg font-bold tracking-tight">
      lab<span class="text-[#6c8fff]">rador</span>
    </h1>
    <button
      class="text-[#7c80a0] text-sm px-3 py-1 rounded-lg border border-[#2e3147] active:opacity-70"
      onclick={() => view.set('prs')}
    >Open PRs</button>
    <button
      class="text-[#7c80a0] p-1 rounded-lg active:opacity-70"
      onclick={() => view.set('settings')}
      aria-label="Settings"
    >
      <svg width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"/>
        <path stroke-linecap="round" stroke-linejoin="round" d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
      </svg>
    </button>
  </header>

  <!-- Capture buttons -->
  <div class="px-4 pt-4 flex gap-3">
    <button
      class="flex-1 flex flex-col items-center gap-2 py-6 rounded-xl border-2 border-dashed border-[#2e3147] bg-[#1a1d27] active:border-[#6c8fff] active:bg-[#6c8fff]/10 transition-colors"
      onclick={openCamera}
    >
      <svg width="32" height="32" fill="none" stroke="#6c8fff" stroke-width="1.5" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" d="M6.827 6.175A2.31 2.31 0 0 1 5.186 7.23c-.38.054-.757.112-1.134.175C2.999 7.58 2.25 8.507 2.25 9.574V18a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9.574c0-1.067-.75-1.994-1.802-2.169a47.865 47.865 0 0 0-1.134-.175 2.31 2.31 0 0 1-1.64-1.055l-.822-1.316a2.192 2.192 0 0 0-1.736-1.039 48.774 48.774 0 0 0-5.232 0 2.192 2.192 0 0 0-1.736 1.039l-.821 1.316z"/>
        <path stroke-linecap="round" stroke-linejoin="round" d="M16.5 12.75a4.5 4.5 0 1 1-9 0 4.5 4.5 0 0 1 9 0z"/>
      </svg>
      <span class="text-[#7c80a0] text-xs">Camera</span>
    </button>

    <label
      for="gallery-input"
      class="flex-1 flex flex-col items-center gap-2 py-6 rounded-xl border-2 border-dashed border-[#2e3147] bg-[#1a1d27] active:border-[#6c8fff] active:bg-[#6c8fff]/10 transition-colors cursor-pointer"
      ondragover={(e) => { e.preventDefault(); dragging = true }}
      ondragleave={() => (dragging = false)}
      ondrop={(e) => { e.preventDefault(); dragging = false; addFiles(e.dataTransfer!.files) }}
    >
      <svg width="32" height="32" fill="none" stroke="#6c8fff" stroke-width="1.5" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" d="m2.25 15.75 5.159-5.159a2.25 2.25 0 0 1 3.182 0l5.159 5.159m-1.5-1.5 1.409-1.409a2.25 2.25 0 0 1 3.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 0 0 1.5-1.5V6a1.5 1.5 0 0 0-1.5-1.5H3.75A1.5 1.5 0 0 0 2.25 6v12a1.5 1.5 0 0 0 1.5 1.5zm10.5-11.25h.008v.008h-.008V8.25zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0z"/>
      </svg>
      <span class="text-[#7c80a0] text-xs">Gallery</span>
    </label>
  </div>

  <input bind:this={galleryInput} id="gallery-input" type="file" accept="image/*" multiple class="hidden"
    onchange={(e) => addFiles((e.target as HTMLInputElement).files!)} />

  <!-- Queue grid -->
  {#if $queue.length > 0}
    <div class="flex-1 overflow-y-auto px-4 pt-4">
      <div class="flex items-center justify-between mb-3">
        <span class="text-[#7c80a0] text-sm">{$queue.length} photo{$queue.length !== 1 ? 's' : ''} queued</span>
        <button class="text-[#ff6b6b] text-xs active:opacity-70" onclick={() => queue.set([])}>Clear all</button>
      </div>
      <div class="grid grid-cols-3 gap-2">
        {#each $queue as img (img.id)}
          <div class="relative aspect-square rounded-xl overflow-hidden border border-[#2e3147]">
            <img src={img.dataUrl} alt="" class="w-full h-full object-cover" />
            <button
              class="absolute top-1.5 right-1.5 w-7 h-7 rounded-full bg-[#ff6b6b] flex items-center justify-center active:opacity-70"
              onclick={() => removeImage(img.id)}
              aria-label="Remove"
            >
              <svg width="14" height="14" fill="none" stroke="white" stroke-width="2.5" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="flex-1"></div>
  {/if}

  <div class="p-4 pb-8">
    <button
      class="w-full py-4 rounded-2xl font-semibold text-base transition-opacity
             {$queue.length === 0 ? 'bg-[#2e3147] text-[#7c80a0] cursor-not-allowed' : 'bg-[#6c8fff] text-white active:opacity-80'}"
      disabled={$queue.length === 0}
      onclick={processQueue}
    >
      Process {$queue.length > 0 ? $queue.length : ''} photo{$queue.length !== 1 ? 's' : ''}
    </button>
  </div>
</div>
