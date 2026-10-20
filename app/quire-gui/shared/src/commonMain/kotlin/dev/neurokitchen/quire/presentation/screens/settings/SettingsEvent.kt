package dev.neurokitchen.quire.presentation.screens.settings

import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme

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
    data class OnSetAppTheme(val theme: AppTheme) : SettingsEvent
    data class OnSetReaderTheme(val theme: ReaderTheme) : SettingsEvent
    data class OnSetReaderFont(val font: ReaderFont) : SettingsEvent
    data class OnSetReaderFontSize(val size: Int) : SettingsEvent
}
