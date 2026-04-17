<script lang="ts">
  import { onMount } from 'svelte'
  import { fly } from 'svelte/transition'
  import { view, githubAuthed, repoConfigured, sharedPayload } from './lib/stores'
  import { cmd } from './lib/tauri'
  import CameraView    from './lib/CameraView.svelte'
  import ProcessingView from './lib/ProcessingView.svelte'
  import PreviewView   from './lib/PreviewView.svelte'
  import DoneView      from './lib/DoneView.svelte'
  import PrsView       from './lib/PrsView.svelte'
  import SettingsView  from './lib/SettingsView.svelte'
  import ShareView     from './lib/ShareView.svelte'

  const VIEW_ORDER = ['camera', 'processing', 'preview', 'done', 'prs', 'settings', 'share']
  let prevView = 'camera'
  let direction = 1   // 1 = slide left (forward), -1 = slide right (back)

  view.subscribe(next => {
    direction = VIEW_ORDER.indexOf(next) >= VIEW_ORDER.indexOf(prevView) ? 1 : -1
    prevView = next
  })

  async function checkPendingShare() {
    try {
      const pending = await cmd.getPendingShare()
      if (pending) {
        sharedPayload.set(pending)
        view.set('share')
      }
    } catch (_) {}
  }

  onMount(async () => {
    // Callback for Android's onNewIntent (app already running when share arrives).
    ;(window as any).__labrador_checkShare = checkPendingShare

    try {
      githubAuthed.set(await cmd.authStatus())
      const repo = await cmd.getRepoConfig()
      repoConfigured.set(!!repo)
      // Send first-time users to settings
      if (!$githubAuthed || !repo) {
        view.set('settings')
        return
      }
      // Check if the app was launched via an Android share intent (cold start)
      await checkPendingShare()
    } catch (_) {}
  })
</script>

<div class="flex flex-col bg-[#0f1117]" style="height: 100dvh">
  <!-- pushes all views below the status bar on every phone -->
  <div class="shrink-0 bg-[#0f1117]" style="height: env(safe-area-inset-top, 0px)"></div>

  <div class="flex-1 overflow-hidden relative">
  {#key $view}
    <div
      class="absolute inset-0"
      in:fly={{ x: direction * 30, duration: 220, opacity: 0 }}
      out:fly={{ x: direction * -30, duration: 180, opacity: 0 }}
    >
      {#if $view === 'camera'}
        <CameraView />
      {:else if $view === 'processing'}
        <ProcessingView />
      {:else if $view === 'preview'}
        <PreviewView />
      {:else if $view === 'done'}
        <DoneView />
      {:else if $view === 'prs'}
        <PrsView />
      {:else if $view === 'settings'}
        <SettingsView />
      {:else if $view === 'share'}
        <ShareView />
      {/if}
    </div>
  {/key}
  </div>
</div>
