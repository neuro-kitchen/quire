package dev.neurokitchen.quire.presentation.screens.settings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.AppInfo
import dev.neurokitchen.quire.domain.model.SettingEntry
import dev.neurokitchen.quire.domain.model.SettingKey
import dev.neurokitchen.quire.domain.model.SettingKind
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.SettingsRepository
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * The config file as a form (one row per key, checked by the engine on apply) and as text. After
 * every save the library is reopened so the engine uses the new values.
 */
class SettingsViewModel(
    private val settings: SettingsRepository,
    private val library: LibraryRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(SettingsState(version = AppInfo.VERSION))
    val state: StateFlow<SettingsState> = _state.asStateFlow()

    private var keys: List<SettingKey> = emptyList()
    private var entries: List<SettingEntry> = emptyList()
    private var checking: Job? = null

    init {
        viewModelScope.launch { load() }
    }

    fun onEvent(event: SettingsEvent) {
        when (event) {
            is SettingsEvent.OnSectionClick -> _state.update {
                it.copy(section = event.section, fields = fields(event.section))
            }
            is SettingsEvent.OnDraftChange -> {
                editField(event.key) { it.copy(draft = event.value, error = null) }
                val field = _state.value.fields.firstOrNull { it.key == event.key }
                if (field != null && (field.kind == SettingKind.Bool || field.kind == SettingKind.Choice)) apply(event.key)
            }
            is SettingsEvent.OnApply -> apply(event.key)
            is SettingsEvent.OnReset -> save(event.key) { settings.unset(event.key) }
            is SettingsEvent.OnRawChange -> {
                _state.update { it.copy(raw = it.raw.copy(text = event.text, error = null)) }
                checking?.cancel()
                checking = viewModelScope.launch {
                    delay(CHECK_DELAY_MS)
                    val problems = settings.check(event.text)
                    _state.update { it.copy(raw = it.raw.copy(problems = problems)) }
                }
            }
            SettingsEvent.OnRawSave -> viewModelScope.launch {
                val text = _state.value.raw.text
                runCatching { settings.writeFile(text) }
                    .onSuccess { afterSave() }
                    .onFailure { e -> _state.update { it.copy(raw = it.raw.copy(error = e.message)) } }
            }
            SettingsEvent.OnRawRevert -> _state.update {
                it.copy(raw = it.raw.copy(text = it.raw.saved, problems = emptyList(), error = null))
            }
        }
    }

    private suspend fun load() {
        runCatching {
            keys = settings.keys()
            entries = settings.entries()
            val text = settings.readFile()
            _state.update {
                it.copy(
                    isLoading = false,
                    configPath = settings.path(),
                    fields = fields(it.section),
                    raw = RawConfigState(text = text, saved = text, problems = settings.check(text)),
                    error = null,
                )
            }
        }.onFailure { e -> _state.update { it.copy(isLoading = false, error = e.message ?: e.toString()) } }
    }

    private fun apply(key: String) {
        val draft = _state.value.fields.firstOrNull { it.key == key }?.draft ?: return
        save(key) { if (draft.isBlank()) settings.unset(key) else settings.set(key, draft.trim()) }
    }

    private fun save(key: String, write: suspend () -> Unit) = viewModelScope.launch {
        runCatching { write() }
            .onSuccess { afterSave() }
            .onFailure { e -> editField(key) { it.copy(error = e.message ?: e.toString()) } }
    }

    private suspend fun afterSave() {
        load()
        library.reopen()
    }

    private fun editField(key: String, change: (SettingField) -> SettingField) = _state.update { state ->
        state.copy(fields = state.fields.map { if (it.key == key) change(it) else it })
    }

    /** The rows of `section`: each fixed key, and each named instance of a `*` key in the file. */
    private fun fields(section: SettingsSection): List<SettingField> {
        val values = entries.associate { it.key to it.value }
        val drafts = _state.value.fields.associateBy { it.key }
        return keys.filter { it.pattern.substringBefore('.') in section.tables }.flatMap { key ->
            val concrete = if ('*' in key.pattern) entries.map { it.key }.filter { matches(key.pattern, it) } else listOf(key.pattern)
            concrete.map { name ->
                val previous = drafts[name]
                SettingField(
                    key = name, help = key.help, kind = key.kind, choices = key.choices, min = key.min, max = key.max,
                    saved = values[name],
                    draft = previous?.takeIf { it.isDirty && it.error != null }?.draft ?: values[name].orEmpty(),
                )
            }
        }
    }

    private companion object {
        const val CHECK_DELAY_MS = 300L

        fun matches(pattern: String, key: String): Boolean {
            val p = pattern.split('.')
            val k = key.split('.')
            return p.size == k.size && p.zip(k).all { (a, b) -> a == "*" || a == b }
        }
    }
}
