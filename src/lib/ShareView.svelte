<script lang="ts">
  import { view, sharedPayload } from './stores'
  import { cmd } from './tauri'

  let submitting = false
  let error = ''
  let title = $sharedPayload?.title ?? ''

  $: url = $sharedPayload?.url ?? ''
  $: domain = (() => { try { return new URL(url).hostname } catch { return '' } })()
  $: faviconSrc = domain ? `https://www.google.com/s2/favicons?domain=${domain}&sz=64` : ''

  async function submit() {
    submitting = true
    error = ''
    try {
      await cmd.submitSharePr($sharedPayload!.url, title)
      sharedPayload.set(null)
      view.set('done')
    } catch (e: any) {
      error = e?.toString() ?? 'Unknown error'
    } finally {
      submitting = false
    }
  }

  function cancel() {
    sharedPayload.set(null)
    view.set('camera')
  }
</script>

<div class="flex flex-col h-full">
  <header class="flex items-center gap-3 px-4 py-3 border-b border-[#2e3147]">
    <button
      class="text-[#7c80a0] active:opacity-70 shrink-0"
      onclick={cancel}
      aria-label="Cancel"
    >
      <svg width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12"/>
      </svg>
    </button>
    <h1 class="text-base font-semibold text-[#e2e4f0]">Save to reading list</h1>
  </header>

  <div class="flex-1 flex flex-col px-4 pt-6 gap-5">
    <!-- Link preview card -->
    <div class="bg-[#1a1d27] border border-[#2e3147] rounded-xl px-3 py-3 flex items-center gap-3">
      {#if faviconSrc}
        <img src={faviconSrc} width="32" height="32" class="rounded-md shrink-0 bg-[#2e3147]" alt={domain} />
      {:else}
        <div class="w-8 h-8 rounded-md bg-[#2e3147] flex items-center justify-center shrink-0">
          <svg width="16" height="16" fill="none" stroke="#6c8fff" stroke-width="2" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" d="M13.19 8.688a4.5 4.5 0 0 1 1.242 7.244l-4.5 4.5a4.5 4.5 0 0 1-6.364-6.364l1.757-1.757m13.35-.622 1.757-1.757a4.5 4.5 0 0 0-6.364-6.364l-4.5 4.5a4.5 4.5 0 0 0 1.242 7.244"/>
          </svg>
        </div>
      {/if}
      <div class="min-w-0">
        {#if domain}
          <p class="text-[#e2e4f0] text-sm font-medium truncate">{domain}</p>
        {/if}
        <p class="text-[#7c80a0] text-xs break-all leading-relaxed {domain ? 'mt-0.5' : ''}">{url}</p>
      </div>
    </div>

    <!-- Editable title -->
    <div>
      <label for="share-title" class="text-[#7c80a0] text-xs uppercase tracking-wider mb-1.5 block">
        Title (optional)
      </label>
      <input
        id="share-title"
        class="w-full bg-[#1a1d27] border border-[#2e3147] rounded-xl px-3 py-2.5 text-sm text-[#e2e4f0] placeholder-[#7c80a0] focus:outline-none focus:border-[#6c8fff] transition-colors"
        bind:value={title}
        placeholder="Leave blank to use 'To Read'"
      />
    </div>

    <!-- Tag preview -->
    <div class="flex items-center gap-2">
      <span class="text-[#7c80a0] text-xs">Tagged as</span>
      <span class="text-[#7c80a0] text-xs bg-[#2e3147] rounded-full px-2 py-0.5">To Read</span>
    </div>
  </div>

  {#if error}
    <p class="text-[#ff6b6b] text-sm px-4 pt-2">{error}</p>
  {/if}

  <div class="p-4 pb-8 flex gap-3">
    <button
      class="flex-1 py-4 rounded-2xl font-semibold text-sm border border-[#2e3147] text-[#7c80a0] active:opacity-70"
      onclick={cancel}
    >Cancel</button>
    <button
      class="flex-1 py-4 rounded-2xl font-semibold text-sm bg-[#6c8fff] text-white active:opacity-80 disabled:opacity-50 flex items-center justify-center gap-2"
      disabled={submitting || !$sharedPayload?.url}
      onclick={submit}
    >
      {#if submitting}
        <div class="w-4 h-4 rounded-full border-2 border-white/30 border-t-white animate-spin"></div>
        Saving…
      {:else}
        Save to Labrador
      {/if}
    </button>
  </div>
</div>
