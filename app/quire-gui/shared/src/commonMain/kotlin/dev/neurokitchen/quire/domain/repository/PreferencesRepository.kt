package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import kotlinx.coroutines.flow.StateFlow

data class GuiPreferences(
    /** "System" (the default) follows the device's light or dark setting. */
    val appTheme: AppTheme = AppTheme.System,
    val readerTheme: ReaderTheme = ReaderTheme.Dark,
    val readerFont: ReaderFont = ReaderFont.Serif,
    val readerFontSize: Int = 16,
    val readerLayout: ReaderLayout = ReaderLayout(),
    /** Recorded reading dates as ISO strings YYYY-MM-DD. */
    val readingDays: Set<String> = emptySet(),
)

/**
 * The reader's pane sizes: the side panes' widths (dp), the figure drawer's side, and the main
 * area's column fractions with two columns ([two]) and with three ([three]).
 */
data class ReaderLayout(
    val annotationsWidth: Int = 260,
    val notesWidth: Int = 280,
    val drawerWidth: Int = 320,
    val drawerOnLeft: Boolean = false,
    val two: List<Float> = listOf(0.5f, 0.5f),
    val three: List<Float> = listOf(0.4f, 0.35f, 0.25f),
)

interface PreferencesRepository {
    val preferences: StateFlow<GuiPreferences>
    suspend fun setAppTheme(theme: AppTheme)
    suspend fun setReaderTheme(theme: ReaderTheme)
    suspend fun setReaderFont(font: ReaderFont)
    suspend fun setReaderFontSize(size: Int)
    suspend fun setReaderLayout(layout: ReaderLayout)
    suspend fun recordReadingToday()
}
