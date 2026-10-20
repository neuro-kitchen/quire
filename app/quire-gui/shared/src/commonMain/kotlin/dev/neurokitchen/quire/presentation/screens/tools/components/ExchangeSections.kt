package dev.neurokitchen.quire.presentation.screens.tools.components

import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.platform.pickFile
import dev.neurokitchen.quire.presentation.platform.pickFolder
import dev.neurokitchen.quire.presentation.platform.pickSaveFile
import dev.neurokitchen.quire.presentation.screens.tools.ToolsEvent
import dev.neurokitchen.quire.presentation.screens.tools.ToolsState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.pick_pdf
import dev.neurokitchen.quire.resources.tools_auto_exports
import dev.neurokitchen.quire.resources.tools_choose_bib
import dev.neurokitchen.quire.resources.tools_choose_export
import dev.neurokitchen.quire.resources.tools_choose_zotero
import dev.neurokitchen.quire.resources.tools_export_hint
import dev.neurokitchen.quire.resources.tools_format
import dev.neurokitchen.quire.resources.tools_import_bib
import dev.neurokitchen.quire.resources.tools_import_into
import dev.neurokitchen.quire.resources.tools_import_none
import dev.neurokitchen.quire.resources.tools_import_pdf
import dev.neurokitchen.quire.resources.tools_import_zotero
import dev.neurokitchen.quire.resources.tools_import_zotero_hint
import dev.neurokitchen.quire.resources.tools_keep_updated
import dev.neurokitchen.quire.resources.tools_no_auto_exports
import dev.neurokitchen.quire.resources.tools_preview
import dev.neurokitchen.quire.resources.tools_preview_count
import dev.neurokitchen.quire.resources.tools_run_now
import dev.neurokitchen.quire.resources.tools_save_as
import dev.neurokitchen.quire.resources.tools_section_export
import dev.neurokitchen.quire.resources.tools_section_import
import dev.neurokitchen.quire.resources.tools_skipped
import dev.neurokitchen.quire.resources.tools_stop
import dev.neurokitchen.quire.resources.tools_whole_library
import dev.neurokitchen.quire.resources.tools_with_subprojects
import dev.neurokitchen.quire.resources.tools_written
import dev.neurokitchen.quire.resources.tools_zotero_result
import org.jetbrains.compose.resources.stringResource

/** Export once (to look at, or to a file) or keep a file up to date; the files kept up to date. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun ExportSection(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    val form = state.export
    SectionHeader(Res.string.tools_section_export, Res.string.tools_export_hint)
    FlowRow(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs), verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        ProjectPicker(state.projects, form.projectPath, Res.string.tools_whole_library) { onEvent(ToolsEvent.OnExportChange(form.copy(projectPath = it))) }
        if (form.projectPath != null) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Checkbox(form.recursive, onCheckedChange = { onEvent(ToolsEvent.OnExportChange(form.copy(recursive = it))) })
                Text(stringResource(Res.string.tools_with_subprojects), style = MaterialTheme.typography.bodyMedium)
            }
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            Hint(stringResource(Res.string.tools_format))
            ChoicePicker(state.formats.ifEmpty { listOf(form.format) }, form.format) { onEvent(ToolsEvent.OnExportChange(form.copy(format = it))) }
        }
    }
    Spacer(Modifier.height(Dimens.SpaceSm))
    val exportTitle = stringResource(Res.string.tools_choose_export)
    val fileName = "references." + extension(form.format)
    FlowRow(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        SecondaryAction(Res.string.tools_preview, enabled = !state.isWorking) { onEvent(ToolsEvent.OnExportPreview) }
        PrimaryAction(Res.string.tools_save_as, enabled = !state.isWorking) { pickSaveFile(exportTitle, fileName)?.let { onEvent(ToolsEvent.OnExportSave(it)) } }
        SecondaryAction(Res.string.tools_keep_updated, enabled = !state.isWorking) { pickSaveFile(exportTitle, fileName)?.let { onEvent(ToolsEvent.OnKeepUpdated(it)) } }
    }
    state.exportPreview?.let { text ->
        Spacer(Modifier.height(Dimens.SpaceSm))
        Hint(stringResource(Res.string.tools_preview_count, state.exportCount))
        Text(
            text.take(PREVIEW_CHARS),
            style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
            color = QuireTheme.colors.fillPrimary,
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(max = 280.dp)
                .clip(RoundedCornerShape(Dimens.RadiusSmall))
                .background(MaterialTheme.colorScheme.surfaceContainerLow)
                .verticalScroll(rememberScrollState())
                .horizontalScroll(rememberScrollState())
                .padding(Dimens.SpaceXs),
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

/** Enough to look at; the file has all of it. */
private const val PREVIEW_CHARS = 20_000

private fun extension(format: String) = when (format) {
    "csl-json" -> "json"
    "hayagriva" -> "yml"
    else -> "bib"
}
