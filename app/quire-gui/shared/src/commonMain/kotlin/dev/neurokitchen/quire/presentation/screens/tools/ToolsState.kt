package dev.neurokitchen.quire.presentation.screens.tools

import androidx.compose.runtime.Immutable
import dev.neurokitchen.quire.domain.model.AutoExport
import dev.neurokitchen.quire.domain.model.LibraryEntry
import dev.neurokitchen.quire.domain.model.ProjectMember
import dev.neurokitchen.quire.domain.model.SyncReport
import dev.neurokitchen.quire.domain.model.ZoteroImport
import dev.neurokitchen.quire.presentation.components.notice.Notice

/** The tools tab's sections, in the order of its side list. */
enum class ToolsSection { Duplicates, Checks, Export, Import, Libraries, Sync }

@Immutable
data class ToolsState(
    val section: ToolsSection = ToolsSection.Duplicates,
    /** An action is running (an online check, an import, a sync): its button waits. */
    val isWorking: Boolean = false,
    val notice: Notice? = null,
    /** An action waiting for the user's confirmation. */
    val confirm: ToolsConfirm? = null,
    /** The library's projects, for the pickers (export, import into, sharing). */
    val projects: List<ProjectChoice> = emptyList(),
    /** `null` until looked for. */
    val duplicates: List<DuplicateGroupState>? = null,
    val checks: List<CheckedState>? = null,
    val retractions: List<RetractionState>? = null,
    val export: ExportForm = ExportForm(),
    val formats: List<String> = emptyList(),
    /** The export as text, to look at before saving, and how many references. */
    val exportPreview: String? = null,
    val exportCount: Int = 0,
    val autoExports: List<AutoExport> = emptyList(),
    /** The project imports go into (`null`: none). */
    val importInto: String? = null,
    val lastZoteroImport: ZoteroImport? = null,
    val libraries: List<LibraryEntry> = emptyList(),
    val sync: SyncState = SyncState(),
)

@Immutable
data class ProjectChoice(val path: String, val name: String)

/** A reference as one line: title, "Creator · Year", cite key. */
@Immutable
data class ReferenceLine(val id: String, val title: String, val creatorYear: String, val citeKey: String)

/** A reference and those that look like the same work; [sharedIdentifier] `null` for a shared title. */
@Immutable
data class DuplicateGroupState(val keep: ReferenceLine, val others: List<DuplicateState>)

@Immutable
data class DuplicateState(val reference: ReferenceLine, val sharedIdentifier: String?)

@Immutable
data class CheckedState(val id: String, val citeKey: String, val issues: List<String>)

/** Notices as "kind: notice (date)"; [error] when the check could not be made. */
@Immutable
data class RetractionState(val reference: ReferenceLine, val notices: List<String>, val error: String?)

/** What to export: a project (with its subprojects if [recursive]) or the library, in a format. */
@Immutable
data class ExportForm(val projectPath: String? = null, val recursive: Boolean = true, val format: String = "biblatex")

@Immutable
data class SyncState(
    val deviceId: String? = null,
    val userId: String? = null,
    /** Other devices: their names (from the settings) and device IDs. */
    val peers: List<PeerState> = emptyList(),
    /** The user key while shown (secret: hidden again by the user). */
    val revealedKey: String? = null,
    val lastReport: SyncReport? = null,
    /** The project whose sharing is shown, and who it is shared with. */
    val sharing: String? = null,
    val members: List<ProjectMember> = emptyList(),
)

@Immutable
data class PeerState(val name: String, val deviceId: String)

/** Actions that cannot be undone from here. */
sealed interface ToolsConfirm {
    /** [other] is removed after its fields, projects, tags, notes, PDFs and highlights move to [keep]. */
    data class Merge(val keep: ReferenceLine, val other: ReferenceLine) : ToolsConfirm
    /** This device takes another device's user key (its identity for sharing). */
    data class ImportKey(val key: String) : ToolsConfirm
}
