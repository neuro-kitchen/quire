package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.domain.model.SettingEntry
import dev.neurokitchen.quire.domain.model.SettingKey

/**
 * The config file (`config.toml`). The engine checks every value and keeps the file's comments
 * and layout; failures throw with its message.
 */
interface SettingsRepository {
    suspend fun path(): String
    suspend fun keys(): List<SettingKey>
    suspend fun entries(): List<SettingEntry>
    suspend fun set(key: String, value: String)
    /** `false` if it was not set. */
    suspend fun unset(key: String): Boolean
    suspend fun readFile(): String
    /** Problems with `text` as a config file (syntax, unknown keys); empty if fine. */
    suspend fun check(text: String): List<String>
    suspend fun writeFile(text: String)
}
