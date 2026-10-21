package dev.neurokitchen.quire.presentation.screens.library.components

import dev.neurokitchen.quire.presentation.designsystem.components.DialogConfirm
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.presentation.designsystem.components.projectChoices
import dev.neurokitchen.quire.presentation.designsystem.components.ChoiceMenu
import dev.neurokitchen.quire.resources.add_remove
import dev.neurokitchen.quire.resources.add_project_none
import dev.neurokitchen.quire.resources.add_project
import dev.neurokitchen.quire.resources.add_file_pick_zotero
import dev.neurokitchen.quire.resources.add_file_pick_bib
import dev.neurokitchen.quire.resources.add_file_hint
import dev.neurokitchen.quire.resources.add_pdf_pick
import dev.neurokitchen.quire.resources.add_pdf_hint
import dev.neurokitchen.quire.resources.add_tab_file
import dev.neurokitchen.quire.resources.add_tab_pdf
import dev.neurokitchen.quire.presentation.platform.FilePicker
import dev.neurokitchen.quire.presentation.platform.PickKind
import dev.neurokitchen.quire.presentation.platform.rememberFilePicker
import androidx.compose.material3.OutlinedButton
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.PrimaryScrollableTabRow
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
    DialogShell(
        title = stringResource(Res.string.add_title),
        subtitle = stringResource(Res.string.add_subtitle),
        icon = Res.drawable.ic_add,
        width = 600.dp,
        onDismiss = { onEvent(LibraryEvent.OnShowAddDialog(false)) },
        actions = {
            DialogCancel({ onEvent(LibraryEvent.OnShowAddDialog(false)) }, stringResource(Res.string.add_cancel))
            DialogConfirm(stringResource(Res.string.add_confirm), { onEvent(LibraryEvent.OnAddConfirm) }, enabled = state.canAdd, working = state.isWorking)
        },
    ) {
        AddReferenceContent(state, onEvent, projects)
    }
}

/**
 * What the add dialog holds, for the phone's full-screen Add too: the project, the ways in as
 * tabs ([scrollableTabs] on narrow screens, so their names do not wrap), the chosen tab, errors.
 */
@Composable
fun AddReferenceContent(
    state: AddDialogState,
    onEvent: (LibraryEvent) -> Unit,
    projects: List<Pair<String, String>>,
    modifier: Modifier = Modifier,
    scrollableTabs: Boolean = false,
) {
    val picker = rememberFilePicker { kind, paths ->
        when (kind) {
            PickKind.Pdfs -> onEvent(LibraryEvent.OnAddPdfsPicked(paths))
            PickKind.Bib, PickKind.Folder -> paths.firstOrNull()?.let { onEvent(LibraryEvent.OnAddFilePicked(it)) }
        }
    }
    Column(modifier) {
        ChoiceMenu(
            label = stringResource(Res.string.add_project),
            selected = state.project,
            choices = projectChoices(stringResource(Res.string.add_project_none), projects),
            onSelect = { onEvent(LibraryEvent.OnAddProjectSelect(it)) },
        )
        Spacer(Modifier.height(Dimens.SpaceSm))
        val tabs: @Composable () -> Unit = {
            AddTab.entries.forEach { tab ->
                Tab(
                    selected = state.tab == tab,
                    onClick = { onEvent(LibraryEvent.OnAddTabSelect(tab)) },
                    text = { Text(stringResource(tab.label()), maxLines = 1) },
                )
            }
        }
        if (scrollableTabs) {
            PrimaryScrollableTabRow(
                selectedTabIndex = state.tab.ordinal,
                containerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                edgePadding = Dimens.SpaceXs,
                modifier = Modifier.clip(RoundedCornerShape(Dimens.RadiusSmall)),
            ) { tabs() }
        } else {
            PrimaryTabRow(
                selectedTabIndex = state.tab.ordinal,
                containerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                modifier = Modifier.clip(RoundedCornerShape(Dimens.RadiusSmall)),
            ) { tabs() }
        }
        Spacer(Modifier.height(Dimens.SpaceMd))
        when (state.tab) {
            AddTab.Identifier -> IdentifierTab(state, onEvent)
            AddTab.Pdf -> PdfTab(state, onEvent, picker)
            AddTab.File -> FileTab(state, onEvent, picker)
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

/** PDF files: each becomes the paper it is. Chosen one at a time; each can be taken off again. */
@Composable
private fun PdfTab(state: AddDialogState, onEvent: (LibraryEvent) -> Unit, picker: FilePicker) {
    Column(verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        Text(stringResource(Res.string.add_pdf_hint), style = MaterialTheme.typography.bodySmall, color = QuireTheme.colors.fillSecondary)
        state.pdfs.forEach { path ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(path.substringAfterLast('/'), style = MaterialTheme.typography.bodyMedium, maxLines = 1, modifier = Modifier.weight(1f))
                TextButton(onClick = { onEvent(LibraryEvent.OnAddPdfRemove(path)) }) { Text(stringResource(Res.string.add_remove)) }
            }
        }
        OutlinedButton(onClick = { picker.pick(PickKind.Pdfs) }) {
            Text(stringResource(Res.string.add_pdf_pick))
        }
    }
}

/** A `.bib` file, or Zotero's data folder. */
@Composable
private fun FileTab(state: AddDialogState, onEvent: (LibraryEvent) -> Unit, picker: FilePicker) {
    val bibTitle = stringResource(Res.string.add_file_pick_bib)
    val zoteroTitle = stringResource(Res.string.add_file_pick_zotero)
    Column(verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        Text(stringResource(Res.string.add_file_hint), style = MaterialTheme.typography.bodySmall, color = QuireTheme.colors.fillSecondary)
        state.file?.let { Text(it, style = MaterialTheme.typography.bodyMedium, maxLines = 2) }
        Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            OutlinedButton(onClick = { picker.pick(PickKind.Bib) }) {
                Text(bibTitle)
            }
            // Zotero's data folder: a desktop's folder (phones pick documents, not folders).
            if (picker.canPickFolders) {
                OutlinedButton(onClick = { picker.pick(PickKind.Folder) }) {
                    Text(zoteroTitle)
                }
            }
        }
    }
}
