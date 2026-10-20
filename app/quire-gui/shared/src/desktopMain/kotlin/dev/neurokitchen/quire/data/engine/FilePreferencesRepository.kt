package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.repository.GuiPreferences
import dev.neurokitchen.quire.domain.repository.PreferencesRepository
import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.withContext
import java.io.File
import java.util.Properties

class FilePreferencesRepository(
    private val preferencesFile: File = defaultPreferencesFile(),
) : PreferencesRepository {

    private val _prefs = MutableStateFlow(load())
    override val preferences: StateFlow<GuiPreferences> = _prefs.asStateFlow()

    private fun load(): GuiPreferences {
        if (!preferencesFile.exists()) return GuiPreferences()
        return try {
            val props = Properties()
            preferencesFile.inputStream().use { props.load(it) }
            val appThemeStr = props.getProperty("app.theme")
            val appTheme = AppTheme.entries.firstOrNull { it.name.equals(appThemeStr, ignoreCase = true) }
                ?: AppTheme.Obsidian
            val readerThemeStr = props.getProperty("reader.theme")
            val readerTheme = ReaderTheme.entries.firstOrNull { it.name.equals(readerThemeStr, ignoreCase = true) }
                ?: ReaderTheme.Dark
            val readerFontStr = props.getProperty("reader.font")
            val readerFont = ReaderFont.entries.firstOrNull { it.name.equals(readerFontStr, ignoreCase = true) }
                ?: ReaderFont.Serif
            val fontSize = props.getProperty("reader.fontSize")?.toIntOrNull() ?: 16
            GuiPreferences(
                appTheme = appTheme,
                readerTheme = readerTheme,
                readerFont = readerFont,
                readerFontSize = fontSize,
            )
        } catch (_: Exception) {
            GuiPreferences()
        }
    }

    private suspend fun save(prefs: GuiPreferences) = withContext(Dispatchers.IO) {
        try {
            preferencesFile.parentFile?.mkdirs()
            val props = Properties()
            props.setProperty("app.theme", prefs.appTheme.name)
            props.setProperty("reader.theme", prefs.readerTheme.name)
            props.setProperty("reader.font", prefs.readerFont.name)
            props.setProperty("reader.fontSize", prefs.readerFontSize.toString())
            preferencesFile.outputStream().use { props.store(it, "Quire GUI Preferences") }
        } catch (_: Exception) {}
    }

    override suspend fun setAppTheme(theme: AppTheme) {
        _prefs.update { it.copy(appTheme = theme) }
        save(_prefs.value)
    }

    override suspend fun setReaderTheme(theme: ReaderTheme) {
        _prefs.update { it.copy(readerTheme = theme) }
        save(_prefs.value)
    }

    override suspend fun setReaderFont(font: ReaderFont) {
        _prefs.update { it.copy(readerFont = font) }
        save(_prefs.value)
    }

    override suspend fun setReaderFontSize(size: Int) {
        _prefs.update { it.copy(readerFontSize = size) }
        save(_prefs.value)
    }

    companion object {
        fun defaultPreferencesFile(): File {
            val userHome = System.getProperty("user.home") ?: "."
            return File(userHome, ".config/quire/gui-preferences.properties")
        }
    }
}
