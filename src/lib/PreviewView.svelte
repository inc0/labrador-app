<script lang="ts">
  import { view, processed, editedMarkdown, editedTitle, pendingImages, queue } from './stores'
  import { cmd } from './tauri'

  let submitting = false
  let error = ''

  $: note = $processed?.note
  $: branch = $processed?.branch ?? ''

  async function submit() {
    if (!$processed) return
    submitting = true
    error = ''
    try {
      const payload = {
        branch,
        title: $editedTitle,
        markdown: $editedMarkdown,
        images: $pendingImages,
        note_source_paths: $processed.note_source_paths,
        supplemental: $processed.supplemental,
      }
      await cmd.submitPr(payload)
      // Clear everything now that PR is submitted
      pendingImages.set([])
      processed.set(null)
      view.set('done')
    } catch (e: any) {
      error = e?.toString() ?? 'Unknown error'
    } finally {
      submitting = false
    }
  }

  function startOver() {
    pendingImages.set([])
    processed.set(null)
    editedMarkdown.set('')
    editedTitle.set('')
    queue.set([])
    view.set('camera')
  }

  let touchStartX = 0
  function onTouchStart(e: TouchEvent) { touchStartX = e.touches[0].clientX }
  function onTouchEnd(e: TouchEvent) {
    const dx = e.changedTouches[0].clientX - touchStartX
    if (touchStartX < 40 && dx > 80) startOver()
  }
</script>

<div class="flex flex-col h-full" ontouchstart={onTouchStart} ontouchend={onTouchEnd}>
  <!-- Header -->
  <header class="flex items-center gap-3 px-4 py-3 border-b border-[#2e3147]">
    <button
      class="text-[#7c80a0] active:opacity-70 shrink-0"
      onclick={startOver}
      aria-label="Back"
    >
      <svg width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5 8.25 12l7.5-7.5"/>
      </svg>
    </button>
    <div class="flex gap-1 flex-wrap flex-1 min-w-0">
      {#each note?.tags ?? [] as tag}
        <span class="text-[#7c80a0] text-xs bg-[#2e3147] rounded-full px-2 py-0.5 shrink-0">{tag}</span>
      {/each}
    </div>
  </header>

  <div class="flex-1 flex flex-col px-4 pt-3 gap-3 overflow-hidden">
    <!-- Editable PR title -->
    <div>
      <label for="pr-title" class="text-[#7c80a0] text-xs uppercase tracking-wider mb-1.5 block">
        PR title
      </label>
      <input
        id="pr-title"
        class="w-full bg-[#1a1d27] border border-[#2e3147] rounded-xl px-3 py-2.5 text-sm text-[#e2e4f0] placeholder-[#7c80a0] focus:outline-none focus:border-[#6c8fff] transition-colors font-medium"
        bind:value={$editedTitle}
        placeholder="Note title"
      />
    </div>

    <!-- Editable markdown -->
    <div class="flex-1 flex flex-col min-h-0">
      <label for="md-editor" class="text-[#7c80a0] text-xs uppercase tracking-wider mb-1.5 block">
        Markdown — edit before submitting
      </label>
      <textarea
        id="md-editor"
        class="flex-1 w-full bg-[#1a1d27] border border-[#2e3147] rounded-xl p-3 font-mono text-sm text-[#e2e4f0] resize-none focus:outline-none focus:border-[#6c8fff] transition-colors leading-relaxed"
        bind:value={$editedMarkdown}
        spellcheck="false"
        autocorrect="off"
        autocapitalize="off"
      ></textarea>
    </div>
  </div>

  {#if error}
    <p class="text-[#ff6b6b] text-sm px-4 pt-2">{error}</p>
  {/if}

  <!-- Actions -->
  <div class="p-4 pb-8 flex gap-3">
    <button
      class="flex-1 py-4 rounded-2xl font-semibold text-sm border border-[#2e3147] text-[#7c80a0] active:opacity-70"
      onclick={startOver}
    >Discard</button>
    <button
      class="flex-1 py-4 rounded-2xl font-semibold text-sm bg-[#6c8fff] text-white active:opacity-80 disabled:opacity-50 flex items-center justify-center gap-2"
      disabled={submitting || !$editedTitle.trim()}
      onclick={submit}
    >
      {#if submitting}
        <div class="w-4 h-4 rounded-full border-2 border-white/30 border-t-white animate-spin"></div>
        Submitting…
      {:else}
        Open PR
      {/if}
    </button>
  </div>
</div>
