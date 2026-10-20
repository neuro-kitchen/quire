package dev.neurokitchen.quire.presentation.mapping

import androidx.compose.runtime.Composable
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_link
import dev.neurokitchen.quire.resources.item_type_book
import dev.neurokitchen.quire.resources.item_type_book_section
import dev.neurokitchen.quire.resources.item_type_conference_paper
import dev.neurokitchen.quire.resources.item_type_journal_article
import dev.neurokitchen.quire.resources.item_type_preprint
import dev.neurokitchen.quire.resources.item_type_report
import dev.neurokitchen.quire.resources.item_type_thesis
import dev.neurokitchen.quire.resources.item_type_webpage
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

/** Display names of the common CSL types; the rest are shown from their CSL name. */
private val labels: Map<String, StringResource> = mapOf(
    "article-journal" to Res.string.item_type_journal_article,
    "paper-conference" to Res.string.item_type_conference_paper,
    "book" to Res.string.item_type_book,
    "chapter" to Res.string.item_type_book_section,
    "article" to Res.string.item_type_preprint,
    "thesis" to Res.string.item_type_thesis,
    "report" to Res.string.item_type_report,
    "webpage" to Res.string.item_type_webpage,
)

/** "Journal Article" for `article-journal`; "Legal case" for an unmapped `legal_case`. */
@Composable
fun kindName(kind: String): String =
    labels[kind]?.let { stringResource(it) }
        ?: kind.replace('-', ' ').replace('_', ' ').replaceFirstChar { it.uppercase() }

fun kindIcon(kind: String): DrawableResource = when (kind) {
    "book", "chapter", "collection" -> Res.drawable.ic_library_books
    "webpage", "post", "post-weblog" -> Res.drawable.ic_link
    else -> Res.drawable.ic_description
}
