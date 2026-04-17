import { writable } from 'svelte/store'

export type View = 'camera' | 'processing' | 'preview' | 'done' | 'settings' | 'prs' | 'share'

export interface QueuedImage {
  id: string
  dataUrl: string   // for display
  data: string      // base64 for Tauri
  mime: string
  file: File
}

export interface QueuedRecording {
  id: string
  durationMs: number
  data: string
  mime: string
}

export interface QueuedText {
  id: string
  text: string
}

export interface SharePayload {
  url: string
  title: string
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
export const recordingsQueue = writable<QueuedRecording[]>([])
export const textQueue       = writable<QueuedText[]>([])
// Snapshot of images taken at process-time, kept until PR is submitted
export const pendingImages   = writable<{ data: string; mime: string }[]>([])
export const processed       = writable<ProcessedNote | null>(null)
export const editedMarkdown  = writable<string>('')
export const editedTitle     = writable<string>('')
export const githubAuthed    = writable<boolean>(false)
export const repoConfigured  = writable<boolean>(false)
export const sharedPayload   = writable<SharePayload | null>(null)
