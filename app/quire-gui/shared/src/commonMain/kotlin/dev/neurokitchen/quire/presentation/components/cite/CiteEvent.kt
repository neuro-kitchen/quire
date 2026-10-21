package dev.neurokitchen.quire.presentation.components.cite

sealed interface CiteEvent {
    data class OnCopy(val id: String, val citeKey: String, val format: CiteFormat) : CiteEvent
    /** The screen put [CiteState.copied] on the clipboard. */
    data object OnCopied : CiteEvent
    data object OnDismissError : CiteEvent
}
