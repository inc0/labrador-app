import { writable } from 'svelte/store'

export type View = 'camera' | 'processing' | 'preview' | 'done' | 'settings' | 'prs'

export interface QueuedImage {
  id: string
  dataUrl: string   // for display
  data: string      // base64 for Tauri
  mime: string
  file: File
}

export interface ProcessedNote {
  note: { title: string; tags: string[]; linked_text: string }
  note_source_paths: string[]
  supplemental: [string, string][]
  markdown: string
  branch: string
}

export interface PrSummary {
  number: number
  title: string
  url: string
  branch: string
  created_at: string
}

export const view            = writable<View>('camera')
export const queue           = writable<QueuedImage[]>([])
// Snapshot of images taken at process-time, kept until PR is submitted
export const pendingImages   = writable<{ data: string; mime: string }[]>([])
export const processed       = writable<ProcessedNote | null>(null)
export const editedMarkdown  = writable<string>('')
export const editedTitle     = writable<string>('')
export const githubAuthed    = writable<boolean>(false)
export const repoConfigured  = writable<boolean>(false)
