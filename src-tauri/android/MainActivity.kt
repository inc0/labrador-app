package dev.labrador.app

import android.content.Intent
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import java.io.File

class MainActivity : TauriActivity() {
  // Saved in onWebViewCreate so we can notify an already-running frontend.
  private var webView: WebView? = null

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    this.webView = webView
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    handleShareIntent(intent, notify = false)
    super.onCreate(savedInstanceState)
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    // App was already running — write the file AND ping the frontend.
    handleShareIntent(intent, notify = true)
  }

  private fun handleShareIntent(intent: Intent?, notify: Boolean) {
    if (intent?.action != Intent.ACTION_SEND) return
    if (intent.type != "text/plain") return
    val sharedText = intent.getStringExtra(Intent.EXTRA_TEXT) ?: return
    val subject = intent.getStringExtra(Intent.EXTRA_SUBJECT) ?: ""
    File(dataDir, "pending_share.txt").writeText("$sharedText\n$subject")
    if (notify) {
      // Trigger the callback registered by App.svelte so it re-reads the file.
      Handler(Looper.getMainLooper()).post {
        webView?.evaluateJavascript(
          "window.__labrador_checkShare && window.__labrador_checkShare()", null
        )
      }
    }
  }
}
