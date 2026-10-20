package dev.neurokitchen.quire.presentation.components.reference_card

sealed interface ReferenceCardEvent {
    data class OnClick(val id: String) : ReferenceCardEvent
}
