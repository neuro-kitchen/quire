package dev.neurokitchen.quire.presentation.screens.tools.components

import dev.neurokitchen.quire.resources.export_menu
import dev.neurokitchen.quire.presentation.components.export_dialog.ExportTarget
import dev.neurokitchen.quire.presentation.components.export_dialog.ExportDialog
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.platform.pickFile
import dev.neurokitchen.quire.presentation.platform.pickFolder
import dev.neurokitchen.quire.presentation.screens.tools.ToolsEvent
import dev.neurokitchen.quire.presentation.screens.tools.ToolsState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.pick_pdf
import dev.neurokitchen.quire.resources.tools_auto_exports
import dev.neurokitchen.quire.resources.tools_choose_bib
import dev.neurokitchen.quire.resources.tools_choose_zotero
import dev.neurokitchen.quire.resources.tools_export_hint
import dev.neurokitchen.quire.resources.tools_import_bib
import dev.neurokitchen.quire.resources.tools_import_into
import dev.neurokitchen.quire.resources.tools_import_none
import dev.neurokitchen.quire.resources.tools_import_pdf
import dev.neurokitchen.quire.resources.tools_import_zotero
import dev.neurokitchen.quire.resources.tools_import_zotero_hint
import dev.neurokitchen.quire.resources.tools_no_auto_exports
import dev.neurokitchen.quire.resources.tools_run_now
import dev.neurokitchen.quire.resources.tools_section_export
import dev.neurokitchen.quire.resources.tools_section_import
import dev.neurokitchen.quire.resources.tools_skipped
import dev.neurokitchen.quire.resources.tools_stop
import dev.neurokitchen.quire.resources.tools_whole_library
import dev.neurokitchen.quire.resources.tools_written
import dev.neurokitchen.quire.resources.tools_zotero_result
import org.jetbrains.compose.resources.stringResource

/** The export dialog (once, or kept up to date), and the files kept up to date. */
@Composable
internal fun ExportSection(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    SectionHeader(Res.string.tools_section_export, Res.string.tools_export_hint)
    PrimaryAction(Res.string.export_menu, enabled = !state.isWorking) { onEvent(ToolsEvent.OnShowExport(true)) }
    if (state.isExportOpen) {
        ExportDialog(
            target = ExportTarget(),
            projects = state.projects.map { it.path to it.name },
            onDismiss = { onEvent(ToolsEvent.OnShowExport(false)) },
            onDone = { onEvent(ToolsEvent.OnExportDone(it)) },
        )
    }
    Spacer(Modifier.height(Dimens.SpaceMd))
    Text(stringResource(Res.string.tools_auto_exports), style = MaterialTheme.typography.titleSmall, color = QuireTheme.colors.fillPrimary)
    Spacer(Modifier.height(Dimens.SpaceXs))
    if (state.autoExports.isEmpty()) Hint(stringResource(Res.string.tools_no_auto_exports))
    state.autoExports.forEach { auto ->
        Group {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(Modifier.weight(1f)) {
                    Text(auto.path, style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace), color = QuireTheme.colors.fillPrimary)
                    Hint(
                        listOfNotNull(
                            auto.projectPath ?: stringResource(Res.string.tools_whole_library),
                            auto.format ?: auto.profile,
                            auto.written?.let { stringResource(Res.string.tools_written, it) },
                        ).joinToString(" · "),
                    )
                }
                SmallAction(Res.string.tools_stop, enabled = !state.isWorking) { onEvent(ToolsEvent.OnStopAutoExport(auto.path)) }
            }
        }
    }
    if (state.autoExports.isNotEmpty()) {
        SecondaryAction(Res.string.tools_run_now, enabled = !state.isWorking) { onEvent(ToolsEvent.OnRunAutoExports) }
    }
}

/** Zotero, `.bib` files and PDFs, into a project if chosen. */
@Composable
internal fun ImportSection(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    SectionHeader(Res.string.tools_section_import)
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        Hint(stringResource(Res.string.tools_import_into))
        ProjectPicker(state.projects, state.importInto, Res.string.tools_import_none) { onEvent(ToolsEvent.OnImportInto(it)) }
    }
    Spacer(Modifier.height(Dimens.SpaceMd))
    val zoteroTitle = stringResource(Res.string.tools_choose_zotero)
    val bibTitle = stringResource(Res.string.tools_choose_bib)
    val pdfTitle = stringResource(Res.string.pick_pdf)
    Group {
        Hint(stringResource(Res.string.tools_import_zotero_hint))
        Spacer(Modifier.height(Dimens.SpaceXs))
        PrimaryAction(Res.string.tools_import_zotero, enabled = !state.isWorking) { pickFolder(zoteroTitle)?.let { onEvent(ToolsEvent.OnImportZotero(it)) } }
        state.lastZoteroImport?.let { z ->
            Spacer(Modifier.height(Dimens.SpaceXs))
            Hint(stringResource(Res.string.tools_zotero_result, z.added, z.existing, z.projects, z.notes, z.pdfs, z.highlights))
            if (z.skipped.isNotEmpty()) Hint(stringResource(Res.string.tools_skipped, z.skipped.joinToString("; ")))
        }
    }
    Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        SecondaryAction(Res.string.tools_import_bib, enabled = !state.isWorking) { pickFile(bibTitle, listOf("bib"))?.let { onEvent(ToolsEvent.OnImportBib(it)) } }
        SecondaryAction(Res.string.tools_import_pdf, enabled = !state.isWorking) { pickFile(pdfTitle, listOf("pdf"))?.let { onEvent(ToolsEvent.OnImportPdf(it)) } }
    }
}
