package dev.neurokitchen.quire.presentation.screens.settings

import androidx.compose.runtime.Immutable
import dev.neurokitchen.quire.domain.model.SettingKind

@Immutable
data class SettingsState(
    val version: String = "",
    val isLoading: Boolean = true,
    /** Where the config file is. */
    val configPath: String = "",
    val section: SettingsSection = SettingsSection.General,
    /** The rows of the selected form section. */
    val fields: List<SettingField> = emptyList(),
    val raw: RawConfigState = RawConfigState(),
    /** Loading failed (the engine's message). */
    val error: String? = null,
)

/** The panes on the left; all but [ConfigFile] are forms over the keys of some tables. */
enum class SettingsSection(val tables: Set<String>) {
    Appearance(emptySet()),
    Reader(setOf("reader")),
    General(setOf("user", "access")),
    Libraries(setOf("library", "libraries")),
    Export(setOf("export")),
    Shortcuts(emptySet()),
    ConfigFile(emptySet()),
}

/** One key's row: what it is, what is saved, what is being typed. */
@Immutable
data class SettingField(
    /** The concrete key (`export.biblatex.format`). */
    val key: String,
    val help: String,
    val kind: SettingKind,
    val choices: List<String> = emptyList(),
    val min: Int? = null,
    val max: Int? = null,
    /** The value in the file, `null` if unset. */
    val saved: String?,
    /** What the field shows (equals `saved` until edited). */
    val draft: String = saved.orEmpty(),
    /** The engine refused the last value. */
    val error: String? = null,
) {
    val isDirty: Boolean get() = draft != saved.orEmpty()
}

@Immutable
data class RawConfigState(
    val text: String = "",
    /** The file as last read or written. */
    val saved: String = "",
    /** Problems the engine found in `text`. */
    val problems: List<String> = emptyList(),
    val error: String? = null,
) {
    val isDirty: Boolean get() = text != saved
}
