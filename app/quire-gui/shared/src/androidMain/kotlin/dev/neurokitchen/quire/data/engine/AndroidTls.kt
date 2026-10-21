package dev.neurokitchen.quire.data.engine

import android.content.Context

/**
 * Gives the engine the app's context, once, before any network call: its HTTPS client checks
 * certificates with Android's own verifier (through the JVM), which needs it.
 * (Rust side: `crates/quire-ffi/src/android.rs`; Kotlin side of the verifier: the
 * `rustls-platform-verifier` dependency.)
 */
internal object AndroidTls {
    private var done = false

    fun initialize(context: Context) {
        if (done) return
        System.loadLibrary("quire_ffi")
        init(context.applicationContext)
        done = true
    }

    @JvmStatic
    private external fun init(context: Context)
}
