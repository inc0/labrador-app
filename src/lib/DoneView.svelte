<script lang="ts">
  import { view, processed, editedMarkdown } from './stores'

  function newNote() {
    processed.set(null)
    editedMarkdown.set('')
    view.set('camera')
  }
</script>

<div class="flex flex-col items-center justify-center h-full gap-6 px-6 text-center">
  <!-- Success icon -->
  <div class="w-20 h-20 rounded-full bg-[#4caf82]/20 flex items-center justify-center">
    <svg width="40" height="40" fill="none" stroke="#4caf82" stroke-width="2.5" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" d="m4.5 12.75 6 6 9-13.5"/>
    </svg>
  </div>

  <div>
    <h2 class="text-xl font-semibold text-[#e2e4f0]">PR opened!</h2>
    <p class="text-[#7c80a0] text-sm mt-1">
      Review, refine, and merge your note on GitHub when ready.
    </p>
  </div>

  <button
    class="w-full py-4 rounded-2xl font-semibold bg-[#6c8fff] text-white active:opacity-80"
    onclick={() => view.set('prs')}
  >
    View open PRs
  </button>
  <button
    class="w-full py-4 rounded-2xl font-semibold border border-[#2e3147] text-[#7c80a0] active:opacity-70"
    onclick={newNote}
  >
    New note
  </button>
</div>
