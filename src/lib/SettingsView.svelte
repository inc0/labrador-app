<script lang="ts">
  import { onMount } from 'svelte'
  import { view, githubAuthed, repoConfigured } from './stores'
  import { cmd } from './tauri'

  // GitHub auth
  let authStep: 'idle' | 'waiting' | 'done' | 'error' = 'idle'
  let userCode = ''
  let verifyUrl = ''
  let authError = ''

  // Gemini key
  let geminiKey = ''
  let geminiSaved = false

  // Repo config
  let repoOwner = ''
  let repoName = ''
  let baseBranch = 'main'
  let repoSaved = false
  let repoError = ''

  // Swipe-right to go back
  let touchStartX = 0
  function onTouchStart(e: TouchEvent) { touchStartX = e.touches[0].clientX }
  function onTouchEnd(e: TouchEvent) {
    const dx = e.changedTouches[0].clientX - touchStartX
    if (touchStartX < 40 && dx > 80) view.set('camera')
  }

  onMount(async () => {
    githubAuthed.set(await cmd.authStatus())
    const key = await cmd.getGeminiKey()
    if (key) geminiKey = key
    const repo = await cmd.getRepoConfig()
    if (repo) { repoOwner = repo.owner; repoName = repo.repo; baseBranch = repo.base_branch }
  })

  async function startAuth() {
    authStep = 'waiting'
    authError = ''
    try {
      const dc = await cmd.authStart()
      userCode = dc.user_code
      verifyUrl = dc.verification_uri
      // Poll in the background — Rust side sleeps between attempts
      cmd.authPoll().then(() => {
        githubAuthed.set(true)
        authStep = 'done'
      }).catch((e: any) => {
        authError = e?.toString() ?? 'Auth failed'
        authStep = 'error'
      })
    } catch (e: any) {
      authError = e?.toString() ?? 'Failed to start auth'
      authStep = 'error'
    }
  }

  async function logout() {
    await cmd.authLogout()
    githubAuthed.set(false)
    authStep = 'idle'
  }

  async function saveGeminiKey() {
    try {
      await cmd.setGeminiKey(geminiKey.trim())
      geminiSaved = true
      setTimeout(() => (geminiSaved = false), 2000)
    } catch (e: any) {
      console.error('Failed to save Gemini key:', e)
    }
  }

  async function saveRepo() {
    repoError = ''
    try {
      await cmd.setRepoConfig(repoOwner.trim(), repoName.trim(), baseBranch.trim() || 'main')
      repoConfigured.set(true)
      repoSaved = true
      setTimeout(() => (repoSaved = false), 2000)
    } catch (e: any) {
      repoError = e?.toString() ?? 'Failed to save'
    }
  }
</script>

<div class="flex flex-col h-full" ontouchstart={onTouchStart} ontouchend={onTouchEnd}>
  <header class="px-4 pt-4 pb-3 border-b border-[#2e3147]">
    <h2 class="text-base font-semibold">Settings</h2>
  </header>

  <div class="flex-1 overflow-y-auto px-4 py-4 flex flex-col gap-6">

    <!-- GitHub auth -->
    <section class="bg-[#1a1d27] border border-[#2e3147] rounded-xl p-4 flex flex-col gap-3">
      <div class="flex items-center justify-between">
        <h3 class="font-semibold text-sm">GitHub</h3>
        {#if $githubAuthed}
          <span class="text-[#4caf82] text-xs bg-[#4caf82]/10 rounded-full px-2 py-0.5">Connected</span>
        {:else}
          <span class="text-[#7c80a0] text-xs">Not connected</span>
        {/if}
      </div>

      {#if $githubAuthed}
        <button
          class="w-full py-2.5 rounded-xl border border-[#ff6b6b]/40 text-[#ff6b6b] text-sm active:opacity-70"
          onclick={logout}
        >Sign out</button>

      {:else if authStep === 'idle' || authStep === 'error'}
        {#if authError}
          <p class="text-[#ff6b6b] text-xs">{authError}</p>
        {/if}
        <button
          class="w-full py-2.5 rounded-xl bg-[#6c8fff] text-white text-sm font-medium active:opacity-80"
          onclick={startAuth}
        >Connect GitHub</button>

      {:else if authStep === 'waiting'}
        <div class="text-center">
          <p class="text-[#7c80a0] text-xs mb-2">Open this URL on any device and enter the code:</p>
          <a href={verifyUrl} class="text-[#6c8fff] text-sm">{verifyUrl}</a>
          <div class="mt-3 py-3 rounded-xl bg-[#0f1117] border border-[#2e3147]">
            <span class="font-mono text-2xl font-bold tracking-[0.3em] text-[#e2e4f0]">{userCode}</span>
          </div>
          <p class="text-[#7c80a0] text-xs mt-2 flex items-center justify-center gap-1">
            <span class="inline-block w-3 h-3 rounded-full border border-[#6c8fff] border-t-transparent animate-spin"></span>
            Waiting for approval…
          </p>
        </div>

      {:else if authStep === 'done'}
        <p class="text-[#4caf82] text-sm text-center">Authenticated successfully!</p>
      {/if}
    </section>

    <!-- Repo config -->
    <section class="bg-[#1a1d27] border border-[#2e3147] rounded-xl p-4 flex flex-col gap-3">
      <h3 class="font-semibold text-sm">Vault repository</h3>
      <input
        class="w-full bg-[#0f1117] border border-[#2e3147] rounded-xl px-3 py-2.5 text-sm text-[#e2e4f0] placeholder-[#7c80a0] focus:outline-none focus:border-[#6c8fff]"
        placeholder="Owner (e.g. my-username)"
        bind:value={repoOwner}
      />
      <input
        class="w-full bg-[#0f1117] border border-[#2e3147] rounded-xl px-3 py-2.5 text-sm text-[#e2e4f0] placeholder-[#7c80a0] focus:outline-none focus:border-[#6c8fff]"
        placeholder="Repo (e.g. labrador-vault)"
        bind:value={repoName}
      />
      <input
        class="w-full bg-[#0f1117] border border-[#2e3147] rounded-xl px-3 py-2.5 text-sm text-[#e2e4f0] placeholder-[#7c80a0] focus:outline-none focus:border-[#6c8fff]"
        placeholder="Base branch (e.g. main)"
        bind:value={baseBranch}
      />
      {#if repoError}
        <p class="text-[#ff6b6b] text-xs">{repoError}</p>
      {/if}
      <button
        class="w-full py-2.5 rounded-xl text-sm font-medium active:opacity-80 transition-colors
               {repoSaved ? 'bg-[#4caf82] text-white' : 'bg-[#6c8fff] text-white'}"
        onclick={saveRepo}
        disabled={!repoOwner || !repoName}
      >{repoSaved ? 'Saved!' : 'Save repo'}</button>
    </section>

    <!-- Gemini key -->
    <section class="bg-[#1a1d27] border border-[#2e3147] rounded-xl p-4 flex flex-col gap-3">
      <h3 class="font-semibold text-sm">Gemini API key</h3>
      <input
        type="password"
        class="w-full bg-[#0f1117] border border-[#2e3147] rounded-xl px-3 py-2.5 text-sm text-[#e2e4f0] placeholder-[#7c80a0] focus:outline-none focus:border-[#6c8fff]"
        placeholder="AIza…"
        bind:value={geminiKey}
      />
      <button
        class="w-full py-2.5 rounded-xl text-sm font-medium active:opacity-80 transition-colors
               {geminiSaved ? 'bg-[#4caf82] text-white' : 'bg-[#6c8fff] text-white'}"
        onclick={saveGeminiKey}
        disabled={!geminiKey}
      >{geminiSaved ? 'Saved!' : 'Save key'}</button>
    </section>

  </div>

  <div class="p-4 pb-safe pb-8">
    <button
      class="w-full py-4 rounded-2xl border border-[#2e3147] text-[#7c80a0] font-semibold text-sm active:opacity-70"
      onclick={() => view.set('camera')}
    >Back to camera</button>
  </div>
</div>
