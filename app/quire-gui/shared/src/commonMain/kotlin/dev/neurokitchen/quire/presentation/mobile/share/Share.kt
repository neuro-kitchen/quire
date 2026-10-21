package dev.neurokitchen.quire.presentation.mobile.share

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/** Something another app shared with Quire (or opened in it): text, or files copied in. */
sealed interface SharedItem {
    data class Text(val text: String) : SharedItem
    /** PDF files, as paths in the app's cache. */
    data class Files(val paths: List<String>) : SharedItem
}

/**
 * Where the platform puts what was shared, for the app to pick up (Android's share sheet and
 * "Open with"): the app opens Add with it, filled in.
 */
object ShareInbox {
    private val _item = MutableStateFlow<SharedItem?>(null)
    val item: StateFlow<SharedItem?> = _item.asStateFlow()

    fun post(item: SharedItem) {
        _item.value = item
    }

    /** The item, once: `null` after the first call. */
    fun take(): SharedItem? = _item.value.also { _item.value = null }
}

private val Url = Regex("""https?://[^\s<>"']+""")
private val Doi = Regex("""\b10\.\d{4,9}/[^\s<>"']+""")
private val Arxiv = Regex("""\barXiv:\s?\d{4}\.\d{4,5}(v\d+)?""", RegexOption.IGNORE_CASE)
private val Pmid = Regex("""\bPMID:?\s*\d{4,9}\b""", RegexOption.IGNORE_CASE)

/**
 * What identifies a paper in shared text, in order: links, DOIs (not inside a link), arXiv IDs,
 * PubMed IDs. A browser shares "title + link", mail and chat share sentences. Without any, the
 * text itself (trimmed), for the user to see and fix.
 */
fun sharedIdentifiers(text: String): List<String> {
    val found = mutableListOf<Pair<Int, String>>()
    val urls = Url.findAll(text).toList()
    urls.forEach { found += it.range.first to it.value.trimEnd('.', ',', ';', ')', ']') }
    fun outsideUrls(range: IntRange) = urls.none { range.first in it.range }
    Doi.findAll(text).filter { outsideUrls(it.range) }.forEach { found += it.range.first to it.value.trimEnd('.', ',', ';', ')', ']') }
    Arxiv.findAll(text).filter { outsideUrls(it.range) }.forEach { found += it.range.first to it.value }
    Pmid.findAll(text).forEach { found += it.range.first to it.value }
    if (found.isEmpty()) return text.trim().takeIf { it.isNotEmpty() }?.let(::listOf).orEmpty()
    return found.sortedBy { it.first }.map { it.second }.distinct()
}
