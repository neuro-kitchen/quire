package dev.neurokitchen.quire.presentation.screens.tools

import dev.neurokitchen.quire.presentation.components.notice.Notice

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

    data class OnShowExport(val open: Boolean) : ToolsEvent
    /** The export dialog saved: closes it and reports what it did. */
    data class OnExportDone(val notice: Notice) : ToolsEvent
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

    /** Shows a code (QR) for a new device, then asks before giving it anything. */
    data object OnLinkDevice : ToolsEvent
    data object OnAllowLink : ToolsEvent
    data object OnRefuseLink : ToolsEvent
    data object OnCancelLink : ToolsEvent
    /** Joins another device's code (scanned or pasted); [name]: this device's, shown there. */
    data class OnJoinLink(val code: String, val name: String) : ToolsEvent
    data class OnAddPeer(val name: String, val deviceId: String) : ToolsEvent
    data class OnRemovePeer(val name: String) : ToolsEvent
    data class OnSyncWith(val peer: String) : ToolsEvent
    data class OnSharingSelect(val projectPath: String?) : ToolsEvent
    data class OnShare(val userId: String, val name: String, val role: String) : ToolsEvent
    data class OnUnshare(val userId: String) : ToolsEvent

    /** Joins the hub of a `quire://hub?…` link: saves its ID and relay, then syncs with its code. */
    data class OnUseHubLink(val link: String) : ToolsEvent
    /** Saves the hub's ID and our own relay (blank: none, n0's relays) and reopens the library. */
    data class OnSaveHub(val hubId: String, val relay: String) : ToolsEvent
    /** Syncs through the hub; [code] (the setup code or an invitation) registers this person once. */
    data class OnSyncHub(val code: String) : ToolsEvent
    data object OnLoadHubDevices : ToolsEvent
    data object OnRevealRecoveryKit : ToolsEvent
    data object OnHideRecoveryKit : ToolsEvent
}
