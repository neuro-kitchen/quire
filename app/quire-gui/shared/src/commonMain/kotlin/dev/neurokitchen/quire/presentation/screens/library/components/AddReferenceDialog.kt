package dev.neurokitchen.quire.presentation.screens.library.components

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
 * Adding references: by identifier (looked up first if wanted), typed in, or pasted BibTeX. Stateless:
 * the library ViewModel does the work, into the project in view.
 */
@Composable
fun AddReferenceDialog(
    state: AddDialogState,
    onEvent: (LibraryEvent) -> Unit,
) {
    Dialog(onDismissRequest = { onEvent(LibraryEvent.OnShowAddDialog(false)) }) {
        Card(
            shape = RoundedCornerShape(Dimens.RadiusLarge),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainer),
            elevation = CardDefaults.cardElevation(defaultElevation = Dimens.ElevationMedium),
            modifier = Modifier.width(520.dp),
        ) {
            Column(Modifier.padding(Dimens.SpaceMd)) {
                Header()
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
        singleLine = true,
        modifier = Modifier.fillMaxWidth(),
        colors = OutlinedTextFieldDefaults.colors(focusedBorderColor = QuireTheme.colors.accent),
    )
    Spacer(Modifier.height(Dimens.SpaceXs))
    TextButton(
        onClick = { onEvent(LibraryEvent.OnLookup) },
        enabled = state.identifier.isNotBlank() && !state.isWorking,
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
    AddTab.Manual -> Res.string.add_tab_manual
    AddTab.Bibtex -> Res.string.add_tab_bibtex
}
