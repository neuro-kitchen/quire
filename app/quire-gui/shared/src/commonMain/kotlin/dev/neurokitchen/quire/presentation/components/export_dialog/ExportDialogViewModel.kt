package dev.neurokitchen.quire.presentation.components.export_dialog

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.domain.model.ExportOptions
import dev.neurokitchen.quire.domain.model.ExportProfile
import dev.neurokitchen.quire.domain.model.ExportSelection
import dev.neurokitchen.quire.domain.repository.ExchangeRepository
import dev.neurokitchen.quire.presentation.components.notice.Notice
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.tools_exported
import dev.neurokitchen.quire.resources.tools_kept_updated
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * The export dialog (Zotero's export, with Better BibTeX's options): what to export, a profile or
 * the options chosen here, the file's encoding; the export as it will be written, updated as the
 * choices change; then saved once or kept updated.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class ExportDialogViewModel(
    target: ExportTarget,
    private val exchange: ExchangeRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(
        ExportDialogState(target = target, formats = exchange.exportFormats()).withEncodings(exchange.exportEncodings()),
    )
    val state: StateFlow<ExportDialogState> = _state.asStateFlow()

    init {
        viewModelScope.launch {
            val profiles = runCatching { exchange.exportProfiles() }.getOrDefault(emptyList())
            _state.update { it.copy(profiles = profiles) }
        }
        // The preview: the latest choices exported, an earlier one still running dropped.
        _state.map { it.selection() }
            .distinctUntilChanged()
            .mapLatest { selection ->
                _state.update { it.copy(isWorking = true) }
                try {
                    val (text, count) = exchange.export(selection)
                    _state.update { it.copy(isWorking = false, preview = text, count = count, error = null) }
                } catch (e: CancellationException) {
                    throw e
                } catch (e: Exception) {
                    _state.update { it.copy(isWorking = false, preview = null, error = e.message ?: e.toString()) }
                }
            }
            .launchIn(viewModelScope)
    }

    fun onEvent(event: ExportDialogEvent) {
        when (event) {
            is ExportDialogEvent.OnProject -> _state.update { it.copy(target = it.target.copy(projectPath = event.projectPath)) }
            is ExportDialogEvent.OnRecursive -> _state.update { it.copy(target = it.target.copy(recursive = event.recursive)) }
            is ExportDialogEvent.OnProfile -> _state.update { state ->
                val profile = state.profiles.firstOrNull { it.name == event.name }
                if (profile == null) state.copy(profile = null) else state.applying(profile)
            }
            is ExportDialogEvent.OnFormat -> custom { it.copy(options = it.options.copy(format = event.format)) }
            is ExportDialogEvent.OnEncoding -> custom { it.copy(encoding = event.encoding) }
            is ExportDialogEvent.OnOmitToggle -> custom {
                it.copy(omitted = if (event.field in it.omitted) it.omitted - event.field else it.omitted + event.field)
            }
            is ExportDialogEvent.OnMonthMacros -> custom { it.copy(options = it.options.copy(monthMacros = event.on)) }
            is ExportDialogEvent.OnPreprintsAsArticles -> custom { it.copy(options = it.options.copy(preprintsAsArticles = event.on)) }
            is ExportDialogEvent.OnSave -> save(event.path, event.keepUpdated)
        }
    }

    private fun save(path: String, keepUpdated: Boolean) {
        val state = _state.value
        if (keepUpdated && !state.canKeepUpdated) return
        _state.update { it.copy(isWorking = true, error = null) }
        viewModelScope.launch {
            try {
                val selection = state.selection()
                val done = if (keepUpdated) {
                    Notice.Done(Res.string.tools_kept_updated, listOf(exchange.keepUpdated(selection, path), path))
                } else {
                    Notice.Done(Res.string.tools_exported, listOf(exchange.exportTo(selection, path), path))
                }
                _state.update { it.copy(isWorking = false, done = done) }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                _state.update { it.copy(isWorking = false, error = e.message ?: e.toString()) }
            }
        }
    }

    /** An option changed here: the options are sent as they are, no longer as the profile. */
    private fun custom(change: (ExportDialogState) -> ExportDialogState) =
        _state.update { change(it).copy(profile = null).withEncodings(exchange.exportEncodings()) }
}

/** The profile's options, shown here, and the profile chosen. */
private fun ExportDialogState.applying(profile: ExportProfile): ExportDialogState {
    val options = profile.options
    return copy(
        profile = profile.name,
        options = options.copy(omit = emptyList(), ascii = false, encoding = ENCODING_UTF8),
        encoding = if (options.ascii) ENCODING_ASCII else options.encoding,
        omitted = OmitField.entries.filter { it.bib in options.omit || it.csl in options.omit }.toSet(),
    )
}

/** The encodings the format can be written in; a chosen one it cannot falls back to UTF-8. */
private fun ExportDialogState.withEncodings(engine: List<String>): ExportDialogState {
    val choices = if (isBib) engine + ENCODING_ASCII else engine.filter { it != ENCODING_WINDOWS_1252 }
    return copy(encodings = choices, encoding = encoding.takeIf { it in choices } ?: ENCODING_UTF8)
}

/** What the engine is asked for: the profile by name, or the options as chosen here. */
private fun ExportDialogState.selection(): ExportSelection {
    val base = ExportSelection(projectPath = target.projectPath, recursive = target.recursive, references = target.references)
    if (profile != null) return base.copy(profile = profile)
    val ascii = encoding == ENCODING_ASCII
    return base.copy(
        options = ExportOptions(
            format = options.format,
            omit = OmitField.entries.filter { it in omitted }.mapNotNull { it.nameIn(options.format) },
            ascii = ascii && isBib,
            monthMacros = options.monthMacros && options.format == "bibtex",
            preprintsAsArticles = options.preprintsAsArticles && isBib,
            encoding = if (ascii) ENCODING_UTF8 else encoding,
        ),
    )
}
