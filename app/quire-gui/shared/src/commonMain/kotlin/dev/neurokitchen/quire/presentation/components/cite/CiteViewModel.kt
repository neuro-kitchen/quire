package dev.neurokitchen.quire.presentation.components.cite

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.domain.model.ExportSelection
import dev.neurokitchen.quire.domain.repository.ExchangeRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/** One reference as a citation to paste: its key (`@key`), or its entry written by the engine. */
class CiteViewModel(private val exchange: ExchangeRepository) : ViewModel() {
    private val _state = MutableStateFlow(CiteState())
    val state: StateFlow<CiteState> = _state.asStateFlow()

    fun onEvent(event: CiteEvent) {
        when (event) {
            is CiteEvent.OnCopy -> copy(event)
            CiteEvent.OnCopied -> _state.update { it.copy(copied = null) }
            CiteEvent.OnDismissError -> _state.update { it.copy(error = null) }
        }
    }

    private fun copy(event: CiteEvent.OnCopy) {
        val format = event.format.engine
        if (format == null) {
            _state.update { it.copy(copied = "@${event.citeKey}", error = null) }
            return
        }
        _state.update { it.copy(working = event.format, error = null) }
        viewModelScope.launch {
            val text = runCatching { exchange.export(ExportSelection(references = listOf(event.id), format = format)).first.trim() }
            _state.update {
                text.fold(
                    onSuccess = { entry -> it.copy(working = null, copied = entry) },
                    onFailure = { e -> it.copy(working = null, error = e.message ?: e.toString()) },
                )
            }
        }
    }
}
