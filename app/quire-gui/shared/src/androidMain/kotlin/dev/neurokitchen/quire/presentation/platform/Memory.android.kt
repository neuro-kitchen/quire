package dev.neurokitchen.quire.presentation.platform

/**
 * An eighth of the app's heap (often 256-512 MB: 32-64 MB), within 16-128 MB. Bitmaps' pixels are
 * outside the heap on current Android, but the system still counts them against the app.
 */
actual fun pageCacheBytes(): Long = (Runtime.getRuntime().maxMemory() / 8).coerceIn(16L * 1024 * 1024, 128L * 1024 * 1024)
