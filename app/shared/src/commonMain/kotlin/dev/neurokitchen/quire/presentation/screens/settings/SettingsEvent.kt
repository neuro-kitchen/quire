package dev.neurokitchen.quire.presentation.screens.settings

sealed interface SettingsEvent {
    data class OnSectionClick(val section: SettingsSection) : SettingsEvent
    data class OnDraftChange(val key: String, val value: String) : SettingsEvent
    /** Save the row's draft (booleans and choices apply on change). */
    data class OnApply(val key: String) : SettingsEvent
    /** Remove the key from the file (back to quire's default). */
    data class OnReset(val key: String) : SettingsEvent
    data class OnRawChange(val text: String) : SettingsEvent
    data object OnRawSave : SettingsEvent
    data object OnRawRevert : SettingsEvent
}
