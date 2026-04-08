<script lang="ts">
  import { onMount } from 'svelte'
  import { queue, view, processed, editedMarkdown, editedTitle, pendingImages } from './stores'
  import { cmd } from './tauri'

  let status = 'Classifying images…'
  let error = ''

  onMount(async () => {
    try {
      const images = $queue.map(img => ({ data: img.data, mime: img.mime }))

      status = 'Transcribing notes…'
      const result = await cmd.processImages(images)

      pendingImages.set(images)
      processed.set(result)
      editedMarkdown.set(result.markdown)
      editedTitle.set(result.note.title)
      queue.set([])
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
