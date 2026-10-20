package dev.neurokitchen.quire.presentation.components.reference_card

import androidx.compose.runtime.Immutable
import dev.neurokitchen.quire.domain.model.Reference

/** One reference in a list or table row. */
@Immutable
data class ReferenceCardState(
    val id: String,
    val citeKey: String,
    /** CSL type, for the icon. */
    val kind: String,
    val title: String,
    /** Zotero's creator column: "Hopfield", "Hodgkin and Huxley", "Hafting et al." */
    val creator: String,
    val year: String,
    val dateAdded: String,
    val hasPdf: Boolean,
    val isSelected: Boolean = false,
) {
    companion object {
        fun from(reference: Reference, isSelected: Boolean = false) = ReferenceCardState(
            id = reference.id,
            citeKey = reference.citeKey,
            kind = reference.kind,
            title = reference.title,
            creator = creatorLabel(reference),
            year = reference.year?.toString() ?: "",
            dateAdded = reference.dateAdded,
            hasPdf = reference.hasPdf,
            isSelected = isSelected,
        )

        fun creatorLabel(reference: Reference): String = reference.creators.let { creators ->
            when (creators.size) {
                0 -> ""
                1 -> creators[0].sortName
                2 -> "${creators[0].sortName} and ${creators[1].sortName}"
                else -> "${creators[0].sortName} et al."
            }
        }
    }
}
