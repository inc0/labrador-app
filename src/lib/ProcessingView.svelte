<script lang="ts">
  import { onMount } from 'svelte'
  import { queue, recordingsQueue, textQueue, view, processed, editedMarkdown, editedTitle, pendingImages } from './stores'
  import { cmd } from './tauri'

  let status = ''
  let error = ''

  onMount(async () => {
    try {
      const images = $queue.map(img => ({ data: img.data, mime: img.mime }))
      const recordings = $recordingsQueue.map(r => ({ data: r.data, mime: r.mime }))
      const texts = $textQueue.map(t => t.text)

      if (recordings.length > 0 && images.length === 0) {
        status = 'Transcribing audio…'
      } else if (recordings.length > 0) {
        status = 'Transcribing audio & classifying images…'
      } else {
        status = 'Classifying images…'
      }

      const result = await cmd.processImages(images, recordings, texts)

      pendingImages.set(images)
      processed.set(result)
      editedMarkdown.set(result.markdown)
      editedTitle.set(result.note.title)
      queue.set([])
      recordingsQueue.set([])
      textQueue.set([])
      view.set('preview')
    } catch (e: any) {
      error = e?.toString() ?? 'Unknown error'
    }
  })
</script>

<div class="flex flex-col items-center justify-center h-full gap-6 px-6">
  {#if error}
    <div class="text-[#ff6b6b] text-center text-sm">
      <p class="text-lg font-semibold mb-2">Something went wrong</p>
      <p class="font-mono">{error}</p>
    </div>
    <button
      class="px-6 py-3 rounded-xl bg-[#2e3147] text-[#e2e4f0] active:opacity-70"
      onclick={() => view.set('camera')}
    >Go back</button>
  {:else}
    <!-- Spinner -->
    <div class="relative w-20 h-20">
      <div class="absolute inset-0 rounded-full border-4 border-[#2e3147]"></div>
      <div class="absolute inset-0 rounded-full border-4 border-transparent border-t-[#6c8fff] animate-spin"></div>
    </div>

    <div class="text-center">
      <p class="text-[#e2e4f0] font-medium">{status}</p>
      <p class="text-[#7c80a0] text-sm mt-1">This may take a moment…</p>
    </div>
  {/if}
</div>
