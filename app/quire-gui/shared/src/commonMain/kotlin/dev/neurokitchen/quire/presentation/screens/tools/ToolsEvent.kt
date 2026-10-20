package dev.neurokitchen.quire.presentation.screens.tools

sealed interface ToolsEvent {
    data class OnSectionSelect(val section: ToolsSection) : ToolsEvent
    data object OnDismissNotice : ToolsEvent
    /** Asks to confirm, or cancels (`null`). */
    data class OnConfirm(val confirm: ToolsConfirm?) : ToolsEvent
    data object OnConfirmed : ToolsEvent

    data object OnFindDuplicates : ToolsEvent
    data class OnDismissDuplicate(val a: String, val b: String) : ToolsEvent

    data object OnCheck : ToolsEvent
    /** Online (Crossref): retraction and correction notices of references with a DOI. */
    data object OnCheckRetractions : ToolsEvent

    data class OnExportChange(val form: ExportForm) : ToolsEvent
    data object OnExportPreview : ToolsEvent
    /** Writes the export to `path` once. */
    data class OnExportSave(val path: String) : ToolsEvent
    /** Writes the export to `path` now and after every change. */
    data class OnKeepUpdated(val path: String) : ToolsEvent
    data class OnStopAutoExport(val path: String) : ToolsEvent
    data object OnRunAutoExports : ToolsEvent

    data class OnImportInto(val projectPath: String?) : ToolsEvent
    /** Zotero's data folder (with `zotero.sqlite` and `storage/`). */
    data class OnImportZotero(val dir: String) : ToolsEvent
    data class OnImportBib(val path: String) : ToolsEvent
    data class OnImportPdf(val path: String) : ToolsEvent

    /** Opens the library `name` (`null`: the default): every tab switches to it. */
    data class OnOpenLibrary(val name: String?) : ToolsEvent
    /** Registers a new library `name` in the usual place. */
    data class OnCreateLibrary(val name: String) : ToolsEvent
    data class OnSetDefaultLibrary(val name: String) : ToolsEvent
    data class OnBackup(val path: String) : ToolsEvent

    data object OnRevealKey : ToolsEvent
    data object OnHideKey : ToolsEvent
    data class OnAddPeer(val name: String, val deviceId: String) : ToolsEvent
    data class OnRemovePeer(val name: String) : ToolsEvent
    data class OnSyncWith(val peer: String) : ToolsEvent
    data class OnSharingSelect(val projectPath: String?) : ToolsEvent
    data class OnShare(val userId: String, val name: String, val role: String) : ToolsEvent
    data class OnUnshare(val userId: String) : ToolsEvent
}
