package dev.neurokitchen.quire.presentation.components.reference_card

sealed interface ReferenceCardEvent {
    data class OnClick(val id: String) : ReferenceCardEvent
    data class OnOpen(val id: String) : ReferenceCardEvent
}
