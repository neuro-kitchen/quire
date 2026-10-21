package dev.neurokitchen.quire.presentation.screens.tools

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.repository.AttachmentRepository
import dev.neurokitchen.quire.domain.repository.CurationRepository
import dev.neurokitchen.quire.domain.repository.ExchangeRepository
import dev.neurokitchen.quire.domain.repository.LibrariesRepository
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.SettingsRepository
import dev.neurokitchen.quire.domain.repository.SyncRepository
import dev.neurokitchen.quire.presentation.components.notice.Notice
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.notice_added
import dev.neurokitchen.quire.resources.notice_already_in_library
import dev.neurokitchen.quire.resources.notice_imported
import dev.neurokitchen.quire.resources.tools_auto_ran
import dev.neurokitchen.quire.resources.tools_auto_stopped
import dev.neurokitchen.quire.resources.tools_backed_up
import dev.neurokitchen.quire.resources.tools_dismissed
import dev.neurokitchen.quire.resources.tools_key_imported
import dev.neurokitchen.quire.resources.tools_library_created
import dev.neurokitchen.quire.resources.tools_library_default
import dev.neurokitchen.quire.resources.tools_library_opened
import dev.neurokitchen.quire.resources.tools_merged
import dev.neurokitchen.quire.resources.tools_peer_added
import dev.neurokitchen.quire.resources.tools_peer_removed
import dev.neurokitchen.quire.resources.tools_shared
import dev.neurokitchen.quire.resources.tools_synced
import dev.neurokitchen.quire.resources.tools_unshared
import dev.neurokitchen.quire.resources.tools_zotero_imported
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * The tools tab: keeping the library clean (duplicates, checks, retractions), getting references in
 * and out (export, imports), the libraries, and sync. Each section loads when shown.
 */
class ToolsViewModel(
    private val library: LibraryRepository,
    private val curation: CurationRepository,
    private val exchange: ExchangeRepository,
    private val attachments: AttachmentRepository,
    private val libraries: LibrariesRepository,
    private val sync: SyncRepository,
    private val settings: SettingsRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(ToolsState())
    val state: StateFlow<ToolsState> = _state.asStateFlow()

    init {
        viewModelScope.launch {
            library.projects().catch { emit(emptyList()) }.collect { projects ->
                _state.update { s -> s.copy(projects = projects.map { ProjectChoice(it.path, it.name) }.sortedBy { it.path }) }
            }
        }
        load(ToolsSection.Duplicates)
    }

    fun onEvent(event: ToolsEvent) {
        when (event) {
            is ToolsEvent.OnSectionSelect -> {
                _state.update { it.copy(section = event.section) }
                load(event.section)
            }
            ToolsEvent.OnDismissNotice -> _state.update { it.copy(notice = null) }
            is ToolsEvent.OnConfirm -> _state.update { it.copy(confirm = event.confirm) }
            ToolsEvent.OnConfirmed -> {
                val confirm = _state.value.confirm ?: return
                _state.update { it.copy(confirm = null) }
                when (confirm) {
                    is ToolsConfirm.Merge -> act {
                        curation.merge(confirm.keep.id, confirm.other.id)
                        loadDuplicates()
                        Notice.Done(Res.string.tools_merged, listOf(confirm.keep.title))
                    }
                    is ToolsConfirm.ImportKey -> act {
                        val userId = sync.importUserKey(confirm.key.trim())
                        loadSync()
                        Notice.Done(Res.string.tools_key_imported, listOf(userId))
                    }
                }
            }

            ToolsEvent.OnFindDuplicates -> act { loadDuplicates(); null }
            is ToolsEvent.OnDismissDuplicate -> act {
                curation.dismissDuplicate(event.a, event.b)
                loadDuplicates()
                Notice.Done(Res.string.tools_dismissed)
            }

            ToolsEvent.OnCheck -> act {
                val checked = curation.check()
                _state.update { s -> s.copy(checks = checked.map { c -> CheckedState(c.id, c.citeKey, c.issues.map { it.message }) }) }
                null
            }
            ToolsEvent.OnCheckRetractions -> act {
                val results = curation.checkRetractions()
                _state.update { s ->
                    s.copy(
                        retractions = results.filter { it.notices.isNotEmpty() || it.error != null }.map { r ->
                            RetractionState(
                                line(r.reference),
                                r.notices.map { n -> listOfNotNull(n.kind, n.notice, n.date?.let { "($it)" }).joinToString(" ") },
                                r.error,
                            )
                        },
                    )
                }
                null
            }

            is ToolsEvent.OnShowExport -> _state.update { it.copy(isExportOpen = event.open) }
            is ToolsEvent.OnExportDone -> {
                _state.update { it.copy(isExportOpen = false, notice = event.notice) }
                viewModelScope.launch { runCatching { loadExport() } }
            }
            is ToolsEvent.OnStopAutoExport -> act {
                exchange.stopAutoExport(event.path)
                loadExport()
                Notice.Done(Res.string.tools_auto_stopped, listOf(event.path))
            }
            ToolsEvent.OnRunAutoExports -> act {
                val runs = exchange.runAutoExports(force = true)
                loadExport()
                Notice.Done(
                    Res.string.tools_auto_ran,
                    listOf(runs.count { it.outcome == "written" }, runs.count { it.outcome == "unchanged" }, runs.count { it.outcome == "failed" }),
                )
            }

            is ToolsEvent.OnImportInto -> _state.update { it.copy(importInto = event.projectPath) }
            is ToolsEvent.OnImportZotero -> act {
                val imported = exchange.importZotero(event.dir, _state.value.importInto)
                _state.update { it.copy(lastZoteroImport = imported) }
                Notice.Done(Res.string.tools_zotero_imported, listOf(imported.added, imported.existing))
            }
            is ToolsEvent.OnImportBib -> act {
                Notice.Done(Res.string.notice_imported, listOf(exchange.importBib(event.path, _state.value.importInto)))
            }
            is ToolsEvent.OnImportPdf -> act {
                val added = attachments.importPdf(event.path, _state.value.importInto).added
                Notice.Done(if (added.isNew) Res.string.notice_added else Res.string.notice_already_in_library, listOf(added.reference.title))
            }

            is ToolsEvent.OnOpenLibrary -> act {
                libraries.open(event.name)
                _state.update { ToolsState(section = it.section) }
                loadLibraries()
                Notice.Done(Res.string.tools_library_opened, listOf(event.name ?: _state.value.libraries.firstOrNull { it.isDefault }?.name.orEmpty()))
            }
            is ToolsEvent.OnCreateLibrary -> act {
                val entry = libraries.register(event.name.trim())
                loadLibraries()
                Notice.Done(Res.string.tools_library_created, listOf(entry.name ?: entry.path))
            }
            is ToolsEvent.OnSetDefaultLibrary -> act {
                libraries.setDefault(event.name)
                loadLibraries()
                Notice.Done(Res.string.tools_library_default, listOf(event.name))
            }
            is ToolsEvent.OnBackup -> act {
                val backup = libraries.backup(event.path)
                Notice.Done(Res.string.tools_backed_up, listOf(backup.files, backup.path))
            }

            ToolsEvent.OnRevealKey -> act {
                val key = sync.exportUserKey()
                _state.update { it.copy(sync = it.sync.copy(revealedKey = key)) }
                null
            }
            ToolsEvent.OnHideKey -> _state.update { it.copy(sync = it.sync.copy(revealedKey = null)) }
            is ToolsEvent.OnAddPeer -> act {
                settings.set(peerKey(event.name.trim()), event.deviceId.trim())
                loadSync()
                Notice.Done(Res.string.tools_peer_added, listOf(event.name.trim()))
            }
            is ToolsEvent.OnRemovePeer -> act {
                settings.unset(peerKey(event.name))
                loadSync()
                Notice.Done(Res.string.tools_peer_removed, listOf(event.name))
            }
            is ToolsEvent.OnSyncWith -> act {
                val report = sync.syncWith(event.peer)
                _state.update { it.copy(sync = it.sync.copy(lastReport = report)) }
                Notice.Done(Res.string.tools_synced, listOf(event.peer, report.sent, report.applied))
            }
            is ToolsEvent.OnSharingSelect -> act {
                val members = event.projectPath?.let { sync.projectMembers(it) }.orEmpty()
                _state.update { it.copy(sync = it.sync.copy(sharing = event.projectPath, members = members)) }
                null
            }
            is ToolsEvent.OnShare -> {
                val path = _state.value.sync.sharing ?: return
                act {
                    sync.shareProject(path, event.userId.trim(), event.name.trim(), event.role)
                    loadMembers(path)
                    Notice.Done(Res.string.tools_shared, listOf(event.name.trim()))
                }
            }
            is ToolsEvent.OnUnshare -> {
                val path = _state.value.sync.sharing ?: return
                act {
                    sync.unshareProject(path, event.userId)
                    val name = _state.value.sync.members.firstOrNull { it.userId == event.userId }?.name ?: event.userId
                    loadMembers(path)
                    Notice.Done(Res.string.tools_unshared, listOf(name))
                }
            }
        }
    }

    /** What a section shows without asking: everything local; online checks wait for a click. */
    private fun load(section: ToolsSection) {
        viewModelScope.launch {
            runCatching {
                when (section) {
                    ToolsSection.Duplicates -> if (_state.value.duplicates == null) loadDuplicates()
                    ToolsSection.Checks -> Unit
                    ToolsSection.Export -> loadExport()
                    ToolsSection.Import -> Unit
                    ToolsSection.Libraries -> loadLibraries()
                    ToolsSection.Sync -> loadSync()
                }
            }.onFailure { e -> if (e is CancellationException) throw e else report(e) }
        }
    }

    private suspend fun loadDuplicates() {
        val groups = curation.duplicates()
        _state.update { s ->
            s.copy(duplicates = groups.map { g -> DuplicateGroupState(line(g.reference), g.duplicates.map { DuplicateState(line(it.reference), it.sharedIdentifier) }) })
        }
    }

    private suspend fun loadExport() {
        val autos = exchange.autoExports()
        _state.update { it.copy(autoExports = autos) }
    }

    private suspend fun loadLibraries() {
        val entries = libraries.libraries()
        _state.update { it.copy(libraries = entries) }
    }

    private suspend fun loadSync() {
        val deviceId = sync.deviceId()
        val userId = sync.userId()
        val peers = settings.entries().mapNotNull { entry -> PEER.matchEntire(entry.key)?.let { PeerState(it.groupValues[1], entry.value) } }
        _state.update { it.copy(sync = it.sync.copy(deviceId = deviceId, userId = userId, peers = peers)) }
    }

    private suspend fun loadMembers(path: String) {
        val members = sync.projectMembers(path)
        _state.update { it.copy(sync = it.sync.copy(members = members)) }
    }

    /** Runs an action with the working mark, then reports its notice (none if `null`) or failure. */
    private fun act(action: suspend () -> Notice?) {
        viewModelScope.launch {
            _state.update { it.copy(isWorking = true) }
            val notice = try {
                action()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Notice.Failed(e.message ?: e.toString())
            }
            _state.update { it.copy(isWorking = false, notice = notice ?: it.notice) }
        }
    }

    private fun report(e: Throwable) = _state.update { it.copy(notice = Notice.Failed(e.message ?: e.toString())) }

    private companion object {
        /** Settings keys of the other devices: `peers.<name>.id` = device ID. */
        val PEER = Regex("""peers\.(.+)\.id""")

        fun peerKey(name: String) = "peers.$name.id"
    }
}

private fun line(reference: Reference): ReferenceLine {
    val card = ReferenceCardState.from(reference)
    return ReferenceLine(reference.id, reference.title, listOf(card.creator, card.year).filter { it.isNotBlank() }.joinToString(" · "), reference.citeKey)
}
