package dev.neurokitchen.quire.domain.model

/** A key the config file accepts; `*` segments are names the user picks (`export.*.format`). */
data class SettingKey(
    val pattern: String,
    val kind: SettingKind,
    /** Words a [SettingKind.Choice] accepts. */
    val choices: List<String> = emptyList(),
    val min: Int? = null,
    val max: Int? = null,
    val help: String,
)

enum class SettingKind { Email, Text, Path, Library, Bool, List, Choice, Number }

/** A value set in the config file, as text (lists comma-separated). */
data class SettingEntry(val key: String, val value: String)
