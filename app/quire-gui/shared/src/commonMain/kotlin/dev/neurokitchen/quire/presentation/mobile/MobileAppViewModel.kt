package dev.neurokitchen.quire.presentation.mobile

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.domain.model.Creator
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.PreferencesRepository
import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.platform.PickKind
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlin.random.Random

data class MobileAppUiState(
    val appTheme: AppTheme = AppTheme.System,
    val showDoiDialog: Boolean = false,
    val showFilePrompt: Boolean = false,
)

sealed interface MobileAppEvent {
    data object OnShowDoiDialog : MobileAppEvent
    data object OnDismissDoiDialog : MobileAppEvent
    data class OnAddByIdentifier(val input: String) : MobileAppEvent
    data object OnShowFilePrompt : MobileAppEvent
    data object OnDismissFilePrompt : MobileAppEvent
    data class OnImportFiles(val kind: PickKind, val paths: List<String>) : MobileAppEvent
}

class MobileAppViewModel(
    private val library: LibraryRepository,
    preferences: PreferencesRepository,
) : ViewModel() {

    private val showDoiDialog = MutableStateFlow(false)
    private val showFilePrompt = MutableStateFlow(false)

    val state: StateFlow<MobileAppUiState> = combine(
        preferences.preferences,
        showDoiDialog,
        showFilePrompt,
    ) { prefs, doiOpen, fileOpen ->
        MobileAppUiState(
            appTheme = prefs.appTheme,
            showDoiDialog = doiOpen,
            showFilePrompt = fileOpen,
        )
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = MobileAppUiState(),
    )

    fun onEvent(event: MobileAppEvent) {
        when (event) {
            MobileAppEvent.OnShowDoiDialog -> showDoiDialog.value = true
            MobileAppEvent.OnDismissDoiDialog -> showDoiDialog.value = false
            is MobileAppEvent.OnAddByIdentifier -> {
                showDoiDialog.value = false
                viewModelScope.launch {
                    library.addByIdentifier(event.input)
                }
            }
            MobileAppEvent.OnShowFilePrompt -> showFilePrompt.value = true
            MobileAppEvent.OnDismissFilePrompt -> showFilePrompt.value = false
            is MobileAppEvent.OnImportFiles -> {
                showFilePrompt.value = false
                importFiles(event.kind, event.paths)
            }
        }
    }

    private fun importFiles(kind: PickKind, paths: List<String>) {
        viewModelScope.launch {
            when (kind) {
                PickKind.Pdfs -> {
                    paths.forEachIndexed { index, path ->
                        val name = path.substringAfterLast('/').removeSuffix(".pdf")
                        val title = name.replace('-', ' ').replace('_', ' ')
                            .split(' ')
                            .filter { it.isNotBlank() }
                            .joinToString(" ") { word ->
                                word.replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
                            }
                        val id = "imported_pdf_${Random.nextInt(1000, 9999)}_$index"
                        val ref = Reference(
                            id = id,
                            citeKey = name.lowercase().filter { it.isLetterOrDigit() }.take(10).ifBlank { "doc_$index" },
                            kind = "article-journal",
                            title = title.ifBlank { "Imported Document" },
                            creators = listOf(Creator(sortName = "Researcher", displayName = "Researcher, A.")),
                            year = 2025,
                            hasPdf = true,
                            dateAdded = "2026-10-22",
                        )
                        library.addReference(ref)
                    }
                }
                PickKind.Bib -> {
                    paths.forEachIndexed { index, path ->
                        val name = path.substringAfterLast('/').removeSuffix(".bib")
                        val id = "imported_bib_${Random.nextInt(1000, 9999)}_$index"
                        val ref = Reference(
                            id = id,
                            citeKey = name.lowercase().filter { it.isLetterOrDigit() }.take(10).ifBlank { "bib_$index" },
                            kind = "article-journal",
                            title = "BibTeX: ${name.replace('-', ' ').replace('_', ' ')}",
                            creators = listOf(Creator(sortName = "Author", displayName = "Author, B.")),
                            year = 2024,
                            hasPdf = false,
                            dateAdded = "2026-10-22",
                        )
                        library.addReference(ref)
                    }
                }
                PickKind.Folder -> Unit
            }
        }
    }
}
