package dev.neurokitchen.quire.data.repository

import dev.neurokitchen.quire.domain.model.SettingEntry
import dev.neurokitchen.quire.domain.model.SettingKey
import dev.neurokitchen.quire.domain.model.SettingKind
import dev.neurokitchen.quire.domain.repository.SettingsRepository

/** Settings kept in memory: tests, previews, platforms without the engine bridge. No checks. */
class SampleSettingsRepository(
    private val keys: List<SettingKey> = KEYS,
    entries: List<SettingEntry> = listOf(SettingEntry("user.name", "Ada"), SettingEntry("reader.width", "72")),
) : SettingsRepository {
    private val values = entries.associate { it.key to it.value }.toMutableMap()
    private var file = ""

    override suspend fun path() = "/home/ada/.config/quire/config.toml"
    override suspend fun keys() = keys
    override suspend fun entries() = values.map { SettingEntry(it.key, it.value) }
    override suspend fun set(key: String, value: String) {
        values[key] = value
    }
    override suspend fun unset(key: String) = values.remove(key) != null
    override suspend fun readFile() = file
    override suspend fun check(text: String) = emptyList<String>()
    override suspend fun writeFile(text: String) {
        file = text
    }

    companion object {
        val KEYS = listOf(
            SettingKey("user.name", SettingKind.Text, help = "your name"),
            SettingKey("user.email", SettingKind.Email, help = "contact email sent to Crossref, NCBI and Unpaywall"),
            SettingKey("export.*.format", SettingKind.Choice, listOf("biblatex", "bibtex"), help = "export profile: output format"),
            SettingKey("reader.width", SettingKind.Number, min = 40, max = 120, help = "reader mode: text column width"),
            SettingKey("reader.justify", SettingKind.Bool, help = "reader mode: justified text"),
            SettingKey("reader.theme", SettingKind.Choice, listOf("dark", "light", "sepia"), help = "reader mode: colors"),
        )
    }
}
