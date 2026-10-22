package dev.neurokitchen.quire.data.repository

import dev.neurokitchen.quire.domain.repository.ReaderLayout
import dev.neurokitchen.quire.domain.repository.GuiPreferences
import dev.neurokitchen.quire.domain.repository.PreferencesRepository
import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

class SamplePreferencesRepository(initial: GuiPreferences = GuiPreferences()) : PreferencesRepository {
    private val _prefs = MutableStateFlow(initial)
    override val preferences: StateFlow<GuiPreferences> = _prefs.asStateFlow()

    override suspend fun setAppTheme(theme: AppTheme) {
        _prefs.update { it.copy(appTheme = theme) }
    }

    override suspend fun setReaderTheme(theme: ReaderTheme) {
        _prefs.update { it.copy(readerTheme = theme) }
    }

    override suspend fun setReaderFont(font: ReaderFont) {
        _prefs.update { it.copy(readerFont = font) }
    }

    override suspend fun setReaderFontSize(size: Int) {
        _prefs.update { it.copy(readerFontSize = size) }
    }

    override suspend fun setReaderLayout(layout: ReaderLayout) {
        _prefs.update { it.copy(readerLayout = layout) }
    }

    override suspend fun recordReadingToday() {
        val today = "2026-10-22"
        _prefs.update { it.copy(readingDays = it.readingDays + today) }
    }
}
