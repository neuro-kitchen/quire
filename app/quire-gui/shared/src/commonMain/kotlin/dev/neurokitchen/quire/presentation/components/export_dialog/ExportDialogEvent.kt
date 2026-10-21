package dev.neurokitchen.quire.presentation.components.export_dialog

sealed interface ExportDialogEvent {
    /** What to export, unless the dialog was opened for a selection. */
    data class OnProject(val projectPath: String?) : ExportDialogEvent
    data class OnRecursive(val recursive: Boolean) : ExportDialogEvent
    data class OnProfile(val name: String?) : ExportDialogEvent
    data class OnFormat(val format: String) : ExportDialogEvent
    data class OnEncoding(val encoding: String) : ExportDialogEvent
    data class OnOmitToggle(val field: OmitField) : ExportDialogEvent
    data class OnMonthMacros(val on: Boolean) : ExportDialogEvent
    data class OnPreprintsAsArticles(val on: Boolean) : ExportDialogEvent
    data class OnSave(val path: String, val keepUpdated: Boolean) : ExportDialogEvent
}
