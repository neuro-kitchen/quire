package dev.neurokitchen.quire.presentation.platform

/** iOS ends apps that use much memory without warning: a modest cache. */
actual fun pageCacheBytes(): Long = 48L * 1024 * 1024
