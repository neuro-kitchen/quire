package dev.neurokitchen.quire.presentation.platform

/**
 * How many bytes of rendered pages the reader keeps: what the platform can spare. The desktop
 * has room for 128 MB; a phone's app heap is far smaller (an eighth of it here).
 */
expect fun pageCacheBytes(): Long
