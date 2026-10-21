package dev.neurokitchen.quire.presentation.components.export_dialog

import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.foundation.clickable
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.presentation.components.notice.Notice
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.Choice
import dev.neurokitchen.quire.presentation.designsystem.components.ChoiceMenu
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.presentation.designsystem.components.projectChoices
import dev.neurokitchen.quire.presentation.platform.pickSaveFile
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.add_cancel
import dev.neurokitchen.quire.resources.export_copy
import dev.neurokitchen.quire.resources.export_encoding
import dev.neurokitchen.quire.resources.export_encoding_ascii
import dev.neurokitchen.quire.resources.export_encoding_utf8
import dev.neurokitchen.quire.resources.export_encoding_utf8_bom
import dev.neurokitchen.quire.resources.export_encoding_windows_1252
import dev.neurokitchen.quire.resources.export_field_abstract
import dev.neurokitchen.quire.resources.export_field_file
import dev.neurokitchen.quire.resources.export_field_keywords
import dev.neurokitchen.quire.resources.export_field_note
import dev.neurokitchen.quire.resources.export_field_url
import dev.neurokitchen.quire.resources.export_format_biblatex
import dev.neurokitchen.quire.resources.export_format_bibtex
import dev.neurokitchen.quire.resources.export_format_csl_json
import dev.neurokitchen.quire.resources.export_format_hayagriva
import dev.neurokitchen.quire.resources.export_keep_updated
import dev.neurokitchen.quire.resources.export_keep_updated_selection
import dev.neurokitchen.quire.resources.export_leave_out
import dev.neurokitchen.quire.resources.export_months
import dev.neurokitchen.quire.resources.export_preprints
import dev.neurokitchen.quire.resources.export_preview
import dev.neurokitchen.quire.resources.export_preview_cut
import dev.neurokitchen.quire.resources.export_profile
import dev.neurokitchen.quire.resources.export_profile_none
import dev.neurokitchen.quire.resources.export_save
import dev.neurokitchen.quire.resources.export_selected
import dev.neurokitchen.quire.resources.export_subtitle
import dev.neurokitchen.quire.resources.export_title
import dev.neurokitchen.quire.resources.export_what
import dev.neurokitchen.quire.resources.ic_upload
import dev.neurokitchen.quire.resources.tools_choose_export
import dev.neurokitchen.quire.resources.tools_format
import dev.neurokitchen.quire.resources.tools_whole_library
import dev.neurokitchen.quire.resources.tools_with_subprojects
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel
import org.koin.core.parameter.parametersOf

/** Characters of the export shown in the preview (a whole library's is megabytes). */
private const val PREVIEW_CHARS = 20_000

/** The labels' width in the options column, so the menus line up. */
private val LABEL_WIDTH = 72.dp

/**
 * The export dialog, for [target] (a project, a selection, the library): its own ViewModel; when
 * saved it closes and [onDone] gets what it did. [projects]: (path, name), to choose another.
 */
@Composable
fun ExportDialog(
    target: ExportTarget,
    projects: List<Pair<String, String>>,
    onDismiss: () -> Unit,
    onDone: (Notice) -> Unit,
    viewModel: ExportDialogViewModel = koinViewModel(key = "export-$target") { parametersOf(target) },
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LaunchedEffect(state.done) { state.done?.let(onDone) }
    ExportDialogContent(state, projects, viewModel::onEvent, onDismiss)
}

/** The dialog, stateless. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ExportDialogContent(
    state: ExportDialogState,
    projects: List<Pair<String, String>>,
    onEvent: (ExportDialogEvent) -> Unit,
    onDismiss: () -> Unit,
) {
    var keepUpdated by remember { mutableStateOf(false) }
    val clipboard = LocalClipboardManager.current
    val saveTitle = stringResource(Res.string.tools_choose_export)
    DialogShell(
        title = stringResource(Res.string.export_title),
        subtitle = stringResource(Res.string.export_subtitle),
        icon = Res.drawable.ic_upload,
        width = 920.dp,
        onDismiss = onDismiss,
        actions = {
            if (state.isWorking) CircularProgressIndicator(Modifier.size(16.dp), strokeWidth = 2.dp)
            TextButton(onClick = onDismiss) { Text(stringResource(Res.string.add_cancel)) }
            OutlinedButton(onClick = { state.preview?.let { clipboard.setText(AnnotatedString(it)) } }, enabled = state.preview != null) {
                Text(stringResource(Res.string.export_copy))
            }
            Button(
                onClick = {
                    pickSaveFile(saveTitle, "references." + extension(state.options.format))?.let {
                        onEvent(ExportDialogEvent.OnSave(it, keepUpdated && state.canKeepUpdated))
                    }
                },
                enabled = state.preview != null && !state.isWorking,
                colors = ButtonDefaults.buttonColors(containerColor = QuireTheme.colors.accent),
            ) { Text(stringResource(Res.string.export_save)) }
        },
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceMd)) {
            Column(Modifier.width(360.dp), verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                if (state.target.references.isNotEmpty()) {
                    Text(stringResource(Res.string.export_selected, state.target.references.size), style = MaterialTheme.typography.bodyMedium)
                } else {
                    ChoiceMenu(
                        label = stringResource(Res.string.export_what),
                        selected = state.target.projectPath,
                        choices = projectChoices(stringResource(Res.string.tools_whole_library), projects),
                        onSelect = { onEvent(ExportDialogEvent.OnProject(it)) },
                        labelWidth = LABEL_WIDTH,
                    )
                    if (state.target.projectPath != null) {
                        LabeledCheckbox(stringResource(Res.string.tools_with_subprojects), state.target.recursive) { onEvent(ExportDialogEvent.OnRecursive(it)) }
                    }
                }
                if (state.profiles.isNotEmpty()) {
                    ChoiceMenu(
                        label = stringResource(Res.string.export_profile),
                        selected = state.profile,
                        choices = listOf(Choice<String?>(null, stringResource(Res.string.export_profile_none))) + state.profiles.map { Choice<String?>(it.name, it.name) },
                        onSelect = { onEvent(ExportDialogEvent.OnProfile(it)) },
                        labelWidth = LABEL_WIDTH,
                    )
                }
                ChoiceMenu(
                    label = stringResource(Res.string.tools_format),
                    selected = state.options.format,
                    choices = state.formats.map { Choice(it, formatName(it)) },
                    onSelect = { onEvent(ExportDialogEvent.OnFormat(it)) },
                    labelWidth = LABEL_WIDTH,
                )
                ChoiceMenu(
                    label = stringResource(Res.string.export_encoding),
                    selected = state.encoding,
                    choices = state.encodings.map { Choice(it, encodingName(it)) },
                    onSelect = { onEvent(ExportDialogEvent.OnEncoding(it)) },
                    labelWidth = LABEL_WIDTH,
                )
                Text(stringResource(Res.string.export_leave_out), style = MaterialTheme.typography.labelMedium, color = QuireTheme.colors.fillSecondary)
                FlowRow(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                    OmitField.entries.filter { it.nameIn(state.options.format) != null }.forEach { field ->
                        FilterChip(
                            selected = field in state.omitted,
                            onClick = { onEvent(ExportDialogEvent.OnOmitToggle(field)) },
                            label = { Text(stringResource(field.label())) },
                        )
                    }
                }
                if (state.options.format == "bibtex") {
                    LabeledCheckbox(stringResource(Res.string.export_months), state.options.monthMacros) { onEvent(ExportDialogEvent.OnMonthMacros(it)) }
                }
                if (state.isBib) {
                    LabeledCheckbox(stringResource(Res.string.export_preprints), state.options.preprintsAsArticles) { onEvent(ExportDialogEvent.OnPreprintsAsArticles(it)) }
                }
                if (state.canKeepUpdated) {
                    LabeledCheckbox(stringResource(Res.string.export_keep_updated), keepUpdated) { keepUpdated = it }
                } else {
                    Text(stringResource(Res.string.export_keep_updated_selection), style = MaterialTheme.typography.bodySmall, color = QuireTheme.colors.fillSecondary)
                }
            }
            Preview(state, Modifier.weight(1f))
        }
    }
}

@Composable
private fun Preview(state: ExportDialogState, modifier: Modifier) {
    Column(modifier) {
        Text(
            stringResource(Res.string.export_preview, state.count),
            style = MaterialTheme.typography.labelMedium,
            color = QuireTheme.colors.fillSecondary,
        )
        Spacer(Modifier.height(Dimens.SpaceXs))
        val text = state.preview
        SelectionContainer(
            Modifier
                .fillMaxWidth()
                .height(420.dp)
                .background(MaterialTheme.colorScheme.surfaceContainerHighest, RoundedCornerShape(Dimens.RadiusSmall))
                .verticalScroll(rememberScrollState())
                .padding(Dimens.SpaceSm),
        ) {
            when {
                state.error != null -> Text(state.error, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error)
                text != null -> Text(
                    if (text.length > PREVIEW_CHARS) text.take(PREVIEW_CHARS) + "\n" + stringResource(Res.string.export_preview_cut, PREVIEW_CHARS) else text,
                    style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
                    color = QuireTheme.colors.fillPrimary,
                )
            }
        }
    }
}

/** A checkbox and its label, the whole row clickable; desktop-sized (no touch-target padding). */
@Composable
private fun LabeledCheckbox(label: String, checked: Boolean, onChange: (Boolean) -> Unit) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
        modifier = Modifier.clickable { onChange(!checked) }.padding(vertical = Dimens.SpaceMin),
    ) {
        CompositionLocalProvider(LocalMinimumInteractiveComponentSize provides 0.dp) {
            Checkbox(checked, onCheckedChange = onChange)
        }
        Text(label, style = MaterialTheme.typography.bodyMedium)
    }
}

@Composable
private fun formatName(format: String): String = when (format) {
    "biblatex" -> stringResource(Res.string.export_format_biblatex)
    "bibtex" -> stringResource(Res.string.export_format_bibtex)
    "csl-json" -> stringResource(Res.string.export_format_csl_json)
    "hayagriva" -> stringResource(Res.string.export_format_hayagriva)
    else -> format
}

@Composable
private fun encodingName(encoding: String): String = when (encoding) {
    ENCODING_UTF8 -> stringResource(Res.string.export_encoding_utf8)
    ENCODING_UTF8_BOM -> stringResource(Res.string.export_encoding_utf8_bom)
    ENCODING_WINDOWS_1252 -> stringResource(Res.string.export_encoding_windows_1252)
    ENCODING_ASCII -> stringResource(Res.string.export_encoding_ascii)
    else -> encoding
}

private fun OmitField.label(): StringResource = when (this) {
    OmitField.Abstract -> Res.string.export_field_abstract
    OmitField.Note -> Res.string.export_field_note
    OmitField.Keywords -> Res.string.export_field_keywords
    OmitField.Url -> Res.string.export_field_url
    OmitField.File -> Res.string.export_field_file
}

/** The usual file extension of a format. */
fun extension(format: String): String = when (format) {
    "csl-json" -> "json"
    "hayagriva" -> "yml"
    else -> "bib"
}
