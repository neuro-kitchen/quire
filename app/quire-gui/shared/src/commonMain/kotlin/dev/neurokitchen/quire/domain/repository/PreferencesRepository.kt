package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import kotlinx.coroutines.flow.StateFlow

data class GuiPreferences(
    val appTheme: AppTheme = AppTheme.Obsidian,
    val readerTheme: ReaderTheme = ReaderTheme.Dark,
    val readerFont: ReaderFont = ReaderFont.Serif,
    val readerFontSize: Int = 16,
)

interface PreferencesRepository {
    val preferences: StateFlow<GuiPreferences>
    suspend fun setAppTheme(theme: AppTheme)
    suspend fun setReaderTheme(theme: ReaderTheme)
    suspend fun setReaderFont(font: ReaderFont)
    suspend fun setReaderFontSize(size: Int)
}
