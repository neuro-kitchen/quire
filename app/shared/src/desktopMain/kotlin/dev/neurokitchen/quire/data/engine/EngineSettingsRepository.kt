package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.SettingEntry
import dev.neurokitchen.quire.domain.model.SettingKey
import dev.neurokitchen.quire.domain.model.SettingKind
import dev.neurokitchen.quire.domain.repository.SettingsRepository
import dev.neurokitchen.quire.ffi.configCheck
import dev.neurokitchen.quire.ffi.configEntries
import dev.neurokitchen.quire.ffi.configKeys
import dev.neurokitchen.quire.ffi.configPath
import dev.neurokitchen.quire.ffi.configRead
import dev.neurokitchen.quire.ffi.configSet
import dev.neurokitchen.quire.ffi.configUnset
import dev.neurokitchen.quire.ffi.configWrite
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** The config file through the engine; `configFile` `null` means the usual one. */
class EngineSettingsRepository(private val configFile: String? = null) : SettingsRepository {
    private suspend fun <T> io(block: () -> T): T = withContext(Dispatchers.IO) { block() }

    override suspend fun path() = io { configPath(configFile) }

    override suspend fun keys() = io {
        configKeys().map {
            SettingKey(
                pattern = it.pattern,
                kind = when (it.kind) {
                    "email" -> SettingKind.Email
                    "path" -> SettingKind.Path
                    "library" -> SettingKind.Library
                    "bool" -> SettingKind.Bool
                    "list" -> SettingKind.List
                    "choice" -> SettingKind.Choice
                    "number" -> SettingKind.Number
                    else -> SettingKind.Text
                },
                choices = it.choices,
                min = it.min?.toInt(),
                max = it.max?.toInt(),
                help = it.help,
            )
        }
    }

    override suspend fun entries() = io { configEntries(configFile).map { SettingEntry(it.key, it.value) } }
    override suspend fun set(key: String, value: String) = io { configSet(configFile, key, value) }
    override suspend fun unset(key: String) = io { configUnset(configFile, key) }
    override suspend fun readFile() = io { configRead(configFile) }
    override suspend fun check(text: String) = io { configCheck(text) }
    override suspend fun writeFile(text: String) = io { configWrite(configFile, text) }
}
