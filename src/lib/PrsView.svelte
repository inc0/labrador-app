<script lang="ts">
  import { onMount } from 'svelte'
  import { view } from './stores'
  import type { PrSummary } from './stores'
  import { cmd } from './tauri'
  import { openUrl } from '@tauri-apps/plugin-opener'

  let prs: PrSummary[] = []
  let loading = true
  let error = ''

  onMount(async () => {
    try {
      prs = await cmd.listPrs()
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load PRs'
    } finally {
      loading = false
    }
  })

  function formatDate(iso: string) {
    return new Date(iso).toLocaleDateString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
  }
</script>

<div class="flex flex-col h-full">
  <header class="flex items-center gap-3 px-4 py-3 border-b border-[#2e3147]">
    <button
      class="text-[#7c80a0] active:opacity-70"
      onclick={() => view.set('camera')}
      aria-label="Back"
    >
      <svg width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5 8.25 12l7.5-7.5"/>
      </svg>
    </button>
    <h2 class="text-base font-semibold">Open note PRs</h2>
  </header>

  <div class="flex-1 overflow-y-auto px-4 py-3">
    {#if loading}
      <div class="flex justify-center pt-12">
        <div class="w-8 h-8 rounded-full border-2 border-[#2e3147] border-t-[#6c8fff] animate-spin"></div>
      </div>
    {:else if error}
      <p class="text-[#ff6b6b] text-sm text-center pt-12">{error}</p>
    {:else if prs.length === 0}
      <p class="text-[#7c80a0] text-sm text-center pt-12">No open PRs</p>
    {:else}
      <div class="flex flex-col gap-3">
        {#each prs as pr}
          <button
            class="w-full text-left bg-[#1a1d27] border border-[#2e3147] rounded-xl p-4 active:border-[#6c8fff] transition-colors"
            onclick={() => openUrl(pr.url)}
          >
            <div class="flex items-start justify-between gap-2">
              <span class="font-medium text-[#e2e4f0] text-sm leading-snug">{pr.title}</span>
              <span class="text-[#6c8fff] text-xs shrink-0">#{pr.number}</span>
            </div>
            <div class="flex items-center gap-2 mt-2">
              <span class="text-[#4caf82] text-xs bg-[#4caf82]/10 rounded-full px-2 py-0.5">open</span>
              <span class="text-[#7c80a0] text-xs">{formatDate(pr.created_at)}</span>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>
