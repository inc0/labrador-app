// Thin wrapper so components don't import Tauri directly — easier to mock in dev.
import { invoke } from '@tauri-apps/api/core'

export const cmd = {
  authStatus:     ()                              => invoke<boolean>('auth_status'),
  authStart:      ()                              => invoke<any>('auth_start'),
  authPoll:       ()                              => invoke<boolean>('auth_poll'),
  authLogout:     ()                              => invoke<void>('auth_logout'),
  setGeminiKey:   (key: string)                   => invoke<void>('set_gemini_key', { key }),
  getGeminiKey:   ()                              => invoke<string | null>('get_gemini_key'),
  setRepoConfig:  (owner: string, repo: string, baseBranch: string) =>
                                                     invoke<void>('set_repo_config', { owner, repo, baseBranch }),
  getRepoConfig:  ()                              => invoke<any>('get_repo_config'),
  processImages:  (images: any[])                 => invoke<any>('process_images', { images }),
  submitPr:       (payload: any)                  => invoke<string>('submit_pr', { payload }),
  listPrs:        ()                              => invoke<any[]>('list_prs'),
}
