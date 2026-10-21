package dev.neurokitchen.quire.presentation.screens.library.components

import dev.neurokitchen.quire.resources.add_remove
import dev.neurokitchen.quire.resources.add_project_none
import dev.neurokitchen.quire.resources.add_project
import dev.neurokitchen.quire.resources.add_file_pick_zotero
import dev.neurokitchen.quire.resources.add_file_pick_bib
import dev.neurokitchen.quire.resources.add_file_hint
import dev.neurokitchen.quire.resources.add_pdf_pick_title
import dev.neurokitchen.quire.resources.add_pdf_pick
import dev.neurokitchen.quire.resources.add_pdf_hint
import dev.neurokitchen.quire.resources.add_tab_file
import dev.neurokitchen.quire.resources.add_tab_pdf
import dev.neurokitchen.quire.resources.ic_expand_more
import dev.neurokitchen.quire.presentation.platform.pickFolder
import dev.neurokitchen.quire.presentation.platform.pickFile
import androidx.compose.runtime.remember
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.getValue
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.DropdownMenu
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.PrimaryTabRow
import androidx.compose.material3.Tab
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.AddDialogState
import dev.neurokitchen.quire.presentation.screens.library.AddTab
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.ManualEntry
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.add_bibtex_label
import dev.neurokitchen.quire.resources.add_cancel
import dev.neurokitchen.quire.resources.add_confirm
import dev.neurokitchen.quire.resources.add_field_authors
import dev.neurokitchen.quire.resources.add_field_container
import dev.neurokitchen.quire.resources.add_field_doi
import dev.neurokitchen.quire.resources.add_field_tags
import dev.neurokitchen.quire.resources.add_field_title
import dev.neurokitchen.quire.resources.add_field_year
import dev.neurokitchen.quire.resources.add_identifier_hint
import dev.neurokitchen.quire.resources.add_identifier_label
import dev.neurokitchen.quire.resources.add_lookup
import dev.neurokitchen.quire.resources.add_subtitle
import dev.neurokitchen.quire.resources.add_tab_bibtex
import dev.neurokitchen.quire.resources.add_tab_identifier
import dev.neurokitchen.quire.resources.add_tab_manual
import dev.neurokitchen.quire.resources.add_title
import dev.neurokitchen.quire.resources.ic_add
import dev.neurokitchen.quire.resources.ic_auto_fix_high
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/**
 * Adding references, every way in one place: identifiers (several, one per line; one can be looked up
 * first), PDF files, a `.bib` file or a Zotero folder, pasted BibTeX, or typed in; into the project
 * chosen at the top. Stateless: the library ViewModel does the work.
 */
@Composable
fun AddReferenceDialog(
    state: AddDialogState,
    onEvent: (LibraryEvent) -> Unit,
    /** Every project as (path, name), for "Add to". */
    projects: List<Pair<String, String>> = emptyList(),
) {
    Dialog(onDismissRequest = { onEvent(LibraryEvent.OnShowAddDialog(false)) }) {
        Card(
            shape = RoundedCornerShape(Dimens.RadiusLarge),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainer),
            elevation = CardDefaults.cardElevation(defaultElevation = Dimens.ElevationMedium),
            modifier = Modifier.width(600.dp),
        ) {
            Column(Modifier.padding(Dimens.SpaceMd)) {
                Header()
                Spacer(Modifier.height(Dimens.SpaceSm))
                ProjectPicker(state.project, projects) { onEvent(LibraryEvent.OnAddProjectSelect(it)) }
                Spacer(Modifier.height(Dimens.SpaceSm))
                PrimaryTabRow(
                    selectedTabIndex = state.tab.ordinal,
                    containerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                    modifier = Modifier.clip(RoundedCornerShape(Dimens.RadiusSmall)),
                ) {
                    AddTab.entries.forEach { tab ->
                        Tab(
                            selected = state.tab == tab,
                            onClick = { onEvent(LibraryEvent.OnAddTabSelect(tab)) },
                            text = { Text(stringResource(tab.label())) },
                        )
                    }
                }
                Spacer(Modifier.height(Dimens.SpaceMd))
                when (state.tab) {
                    AddTab.Identifier -> IdentifierTab(state, onEvent)
                    AddTab.Pdf -> PdfTab(state, onEvent)
                    AddTab.File -> FileTab(state, onEvent)
                    AddTab.Manual -> ManualTab(state.manual) { onEvent(LibraryEvent.OnManualChange(it)) }
                    AddTab.Bibtex -> OutlinedTextField(
                        value = state.bibtex,
                        onValueChange = { onEvent(LibraryEvent.OnBibtexChange(it)) },
                        label = { Text(stringResource(Res.string.add_bibtex_label)) },
                        placeholder = { Text("@article{key,\n  title = {…},\n  author = {…},\n}") },
                        modifier = Modifier.fillMaxWidth().height(160.dp),
                        textStyle = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
                    )
                }
                state.error?.let { error ->
                    Spacer(Modifier.height(Dimens.SpaceXs))
                    Text(error, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error)
                }
                Spacer(Modifier.height(Dimens.SpaceMd))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    TextButton(onClick = { onEvent(LibraryEvent.OnShowAddDialog(false)) }) {
                        Text(stringResource(Res.string.add_cancel))
                    }
                    Spacer(Modifier.width(Dimens.SpaceXs))
                    Button(
                        onClick = { onEvent(LibraryEvent.OnAddConfirm) },
                        colors = ButtonDefaults.buttonColors(containerColor = QuireTheme.colors.accent),
                        enabled = state.canAdd,
                    ) {
                        if (state.isWorking) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(16.dp),
                                strokeWidth = 2.dp,
                                color = MaterialTheme.colorScheme.onPrimary,
                            )
                            Spacer(Modifier.width(Dimens.SpaceXs))
                        }
                        Text(stringResource(Res.string.add_confirm))
                    }
                }
            }
        }
    }
}

@Composable
private fun Header() {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        Box(
            modifier = Modifier
                .size(36.dp)
                .clip(RoundedCornerShape(Dimens.RadiusBase))
                .background(QuireTheme.colors.accent.copy(alpha = 0.15f)),
            contentAlignment = Alignment.Center,
        ) {
            Icon(painterResource(Res.drawable.ic_add), contentDescription = null, tint = QuireTheme.colors.accent, modifier = Modifier.size(20.dp))
        }
        Column {
            Text(
                stringResource(Res.string.add_title),
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.SemiBold,
                color = QuireTheme.colors.fillPrimary,
            )
            Text(stringResource(Res.string.add_subtitle), style = MaterialTheme.typography.bodySmall, color = QuireTheme.colors.fillSecondary)
        }
    }
}

@Composable
private fun IdentifierTab(state: AddDialogState, onEvent: (LibraryEvent) -> Unit) {
    OutlinedTextField(
        value = state.identifier,
        onValueChange = { onEvent(LibraryEvent.OnIdentifierChange(it)) },
        label = { Text(stringResource(Res.string.add_identifier_label)) },
        placeholder = { Text(stringResource(Res.string.add_identifier_hint)) },
        minLines = 3,
        maxLines = 8,
        modifier = Modifier.fillMaxWidth(),
        colors = OutlinedTextFieldDefaults.colors(focusedBorderColor = QuireTheme.colors.accent),
    )
    Spacer(Modifier.height(Dimens.SpaceXs))
    TextButton(
        onClick = { onEvent(LibraryEvent.OnLookup) },
        // A preview is for one identifier; several are added as they are.
        enabled = state.identifier.isNotBlank() && state.identifier.trim().lines().size == 1 && !state.isWorking,
    ) {
        Icon(painterResource(Res.drawable.ic_auto_fix_high), contentDescription = null, modifier = Modifier.size(16.dp))
        Spacer(Modifier.width(Dimens.SpaceXs))
        Text(stringResource(Res.string.add_lookup))
    }
    state.preview?.let { work ->
        Spacer(Modifier.height(Dimens.SpaceSm))
        Card(
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHighest),
            shape = RoundedCornerShape(Dimens.RadiusSmall),
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(Modifier.padding(Dimens.SpaceSm)) {
                Text(work.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
                Text(
                    listOf(work.creator, work.year).filter { it.isNotBlank() }.joinToString(" · "),
                    style = MaterialTheme.typography.bodySmall,
                    color = QuireTheme.colors.fillSecondary,
                )
            }
        }
    }
}

@Composable
private fun ManualTab(entry: ManualEntry, onChange: (ManualEntry) -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        OutlinedTextField(
            value = entry.title,
            onValueChange = { onChange(entry.copy(title = it)) },
            label = { Text(stringResource(Res.string.add_field_title)) },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            OutlinedTextField(
                value = entry.authors,
                onValueChange = { onChange(entry.copy(authors = it)) },
                label = { Text(stringResource(Res.string.add_field_authors)) },
                singleLine = true,
                modifier = Modifier.weight(2f),
            )
            OutlinedTextField(
                value = entry.year,
                onValueChange = { onChange(entry.copy(year = it)) },
                label = { Text(stringResource(Res.string.add_field_year)) },
                singleLine = true,
                modifier = Modifier.weight(1f),
            )
        }
        Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            OutlinedTextField(
                value = entry.containerTitle,
                onValueChange = { onChange(entry.copy(containerTitle = it)) },
                label = { Text(stringResource(Res.string.add_field_container)) },
                singleLine = true,
                modifier = Modifier.weight(1f),
            )
            OutlinedTextField(
                value = entry.doi,
                onValueChange = { onChange(entry.copy(doi = it)) },
                label = { Text(stringResource(Res.string.add_field_doi)) },
                singleLine = true,
                modifier = Modifier.weight(1f),
            )
        }
        OutlinedTextField(
            value = entry.tags,
            onValueChange = { onChange(entry.copy(tags = it)) },
            label = { Text(stringResource(Res.string.add_field_tags)) },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

private fun AddTab.label() = when (this) {
    AddTab.Identifier -> Res.string.add_tab_identifier
    AddTab.Pdf -> Res.string.add_tab_pdf
    AddTab.File -> Res.string.add_tab_file
    AddTab.Manual -> Res.string.add_tab_manual
    AddTab.Bibtex -> Res.string.add_tab_bibtex
}

/** "Add to": no project, or one of the library's (indented by depth). */
@Composable
private fun ProjectPicker(selected: String?, projects: List<Pair<String, String>>, onSelect: (String?) -> Unit) {
    var open by remember { mutableStateOf(false) }
    val none = stringResource(Res.string.add_project_none)
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        Text(stringResource(Res.string.add_project), style = MaterialTheme.typography.labelMedium, color = QuireTheme.colors.fillSecondary)
        Box {
            OutlinedButton(onClick = { open = true }) {
                Text(projects.firstOrNull { it.first == selected }?.second ?: none, maxLines = 1)
                Icon(painterResource(Res.drawable.ic_expand_more), contentDescription = null, modifier = Modifier.size(16.dp))
            }
            DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
                DropdownMenuItem(text = { Text(none) }, onClick = { open = false; onSelect(null) })
                projects.forEach { (path, name) ->
                    val depth = path.count { it == '/' }
                    DropdownMenuItem(
                        text = { Text(name, modifier = Modifier.padding(start = Dimens.SpaceSm * depth)) },
                        onClick = { open = false; onSelect(path) },
                    )
                }
            }
        }
    }
}

/** PDF files: each becomes the paper it is. Chosen one at a time; each can be taken off again. */
@Composable
private fun PdfTab(state: AddDialogState, onEvent: (LibraryEvent) -> Unit) {
    val title = stringResource(Res.string.add_pdf_pick_title)
    Column(verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        Text(stringResource(Res.string.add_pdf_hint), style = MaterialTheme.typography.bodySmall, color = QuireTheme.colors.fillSecondary)
        state.pdfs.forEach { path ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(path.substringAfterLast('/'), style = MaterialTheme.typography.bodyMedium, maxLines = 1, modifier = Modifier.weight(1f))
                TextButton(onClick = { onEvent(LibraryEvent.OnAddPdfRemove(path)) }) { Text(stringResource(Res.string.add_remove)) }
            }
        }
        OutlinedButton(onClick = { pickFile(title, listOf("pdf"))?.let { onEvent(LibraryEvent.OnAddPdfsPicked(listOf(it))) } }) {
            Text(stringResource(Res.string.add_pdf_pick))
        }
    }
}

/** A `.bib` file, or Zotero's data folder. */
@Composable
private fun FileTab(state: AddDialogState, onEvent: (LibraryEvent) -> Unit) {
    val bibTitle = stringResource(Res.string.add_file_pick_bib)
    val zoteroTitle = stringResource(Res.string.add_file_pick_zotero)
    Column(verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        Text(stringResource(Res.string.add_file_hint), style = MaterialTheme.typography.bodySmall, color = QuireTheme.colors.fillSecondary)
        state.file?.let { Text(it, style = MaterialTheme.typography.bodyMedium, maxLines = 2) }
        Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            OutlinedButton(onClick = { pickFile(bibTitle, listOf("bib"))?.let { onEvent(LibraryEvent.OnAddFilePicked(it)) } }) {
                Text(bibTitle)
            }
            OutlinedButton(onClick = { pickFolder(zoteroTitle)?.let { onEvent(LibraryEvent.OnAddFilePicked(it)) } }) {
                Text(zoteroTitle)
            }
        }
    }
}
