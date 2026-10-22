package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.draw.clip
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.InputChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.components.notice.NoticeBar
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryState
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.library.ReferenceDetailState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.details_add
import dev.neurokitchen.quire.resources.details_add_note
import dev.neurokitchen.quire.resources.details_add_tag
import dev.neurokitchen.quire.resources.details_add_to_project
import dev.neurokitchen.quire.resources.details_more
import dev.neurokitchen.quire.resources.details_publisher_page
import dev.neurokitchen.quire.resources.details_read
import dev.neurokitchen.quire.resources.details_remove
import dev.neurokitchen.quire.resources.details_show_less
import dev.neurokitchen.quire.resources.details_show_more
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_more_vert
import dev.neurokitchen.quire.resources.item_delete
import dev.neurokitchen.quire.resources.item_get_pdf
import dev.neurokitchen.quire.resources.item_refetch
import dev.neurokitchen.quire.resources.item_restore
import dev.neurokitchen.quire.resources.reading_read
import dev.neurokitchen.quire.resources.reading_reading
import dev.neurokitchen.quire.resources.reading_unread
import dev.neurokitchen.quire.resources.section_abstract
import dev.neurokitchen.quire.resources.section_attachments
import dev.neurokitchen.quire.resources.section_empty_attachments
import dev.neurokitchen.quire.resources.section_empty_notes
import dev.neurokitchen.quire.resources.section_info
import dev.neurokitchen.quire.resources.section_notes
import dev.neurokitchen.quire.resources.section_projects
import dev.neurokitchen.quire.resources.section_tags
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/** An abstract longer than this many characters is folded, with "Show more". */
private const val ABSTRACT_FOLD = 320

/**
 * A paper's details on a phone: what it is, Read (or Get PDF), its reading status, then its
 * abstract, details, tags, projects, notes and files; less common actions in the overflow menu.
 */
@Composable
fun DetailsScreen(
    id: String,
    onBack: () -> Unit,
    onOpenReader: (String, String) -> Unit,
    viewModel: LibraryViewModel = koinViewModel(),
) {
    LaunchedEffect(id) { viewModel.onEvent(LibraryEvent.OnReferenceClick(id)) }
    val state by viewModel.state.collectAsStateWithLifecycle()
    val onEvent = viewModel::onEvent
    val chrome = LocalMobileChrome.current
    val uriHandler = LocalUriHandler.current
    LaunchedEffect(state.pageToOpen) {
        state.pageToOpen?.let { page ->
            runCatching { uriHandler.openUri(page) }
            onEvent(LibraryEvent.OnPageOpened)
        }
    }
    val reference = state.selected
    Box(Modifier.fillMaxSize()) {
        if (reference != null) {
            LazyColumn(
                Modifier.fillMaxSize().hazeSource(chrome.haze),
                contentPadding = mobileContentPadding(),
                verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
            ) {
                item {
                    Header(
                        reference = reference,
                        isFetchingPdf = state.isFetchingPdf,
                        onRead = { onOpenReader(reference.id, reference.title) },
                        onGetPdf = { onEvent(LibraryEvent.OnGetPdf) },
                    )
                }
                item { Status(reference) { onEvent(LibraryEvent.OnBatchSetReadingStatus(setOf(reference.id), it)) } }
                if (reference.abstract.isNotBlank()) item { AbstractSection(reference.abstract) }
                if (reference.fields.isNotEmpty()) item { InfoSection(reference) }
                item { TagsSection(reference, state, onEvent) }
                item { ProjectsSection(reference, state, onEvent) }
                item { NotesSection(reference, state, onEvent) }
                item { FilesSection(reference) }
            }
        }
        MobileTopBar("", onBack, chrome.haze, actions = {
            if (reference != null) Overflow(reference, onEvent, onBack)
        })
        state.notice?.let { notice ->
            NoticeBar(
                notice,
                onDismiss = { onEvent(LibraryEvent.OnDismissNotice) },
                modifier = Modifier.align(Alignment.BottomCenter).padding(Dimens.SpaceMd).padding(bottom = chrome.bottomBar),
            )
        }
    }
}

@Composable
private fun Header(
    reference: ReferenceDetailState,
    isFetchingPdf: Boolean,
    onRead: () -> Unit,
    onGetPdf: () -> Unit,
) {
    val colors = QuireTheme.colors
    Column(Modifier.fillMaxWidth().padding(horizontal = Dimens.SpaceLg), verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        Text(
            reference.title,
            style = MaterialTheme.typography.headlineSmall.copy(fontFamily = FontFamily.Serif),
            fontWeight = FontWeight.SemiBold,
            color = colors.fillPrimary,
        )
        val authors = reference.authors.joinToString("; ").ifBlank { reference.creator }
        if (authors.isNotBlank()) Text(authors, style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary, maxLines = 3, overflow = TextOverflow.Ellipsis)
        Text(reference.year, style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary)
        Spacer(Modifier.size(Dimens.SpaceXs))
        if (!reference.hasPdf) {
            // Material 3 Expressive Split Button with clear separation
            Row(
                modifier = Modifier.fillMaxWidth().height(48.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceMin),
            ) {
                Button(
                    onClick = onRead,
                    modifier = Modifier.weight(1f).fillMaxHeight(),
                    shape = RoundedCornerShape(topStart = 24.dp, bottomStart = 24.dp, topEnd = 8.dp, bottomEnd = 8.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = colors.accent, contentColor = colors.onAccent),
                    contentPadding = PaddingValues(horizontal = Dimens.SpaceMd),
                ) {
                    Text(
                        stringResource(Res.string.details_read),
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
                Button(
                    onClick = onGetPdf,
                    enabled = !isFetchingPdf,
                    modifier = Modifier.fillMaxHeight(),
                    shape = RoundedCornerShape(topStart = 8.dp, bottomStart = 8.dp, topEnd = 24.dp, bottomEnd = 24.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = colors.accent, contentColor = colors.onAccent),
                    contentPadding = PaddingValues(horizontal = Dimens.SpaceMd),
                ) {
                    if (isFetchingPdf) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(18.dp),
                            color = colors.onAccent,
                            strokeWidth = 2.dp,
                        )
                    } else {
                        Text(
                            stringResource(Res.string.item_get_pdf),
                            style = MaterialTheme.typography.labelLarge,
                            fontWeight = FontWeight.SemiBold,
                        )
                    }
                }
            }
        } else {
            Button(
                onClick = onRead,
                colors = ButtonDefaults.buttonColors(containerColor = colors.accent, contentColor = colors.onAccent),
                shape = RoundedCornerShape(24.dp),
                modifier = Modifier.fillMaxWidth().height(48.dp),
            ) {
                Text(
                    stringResource(Res.string.details_read),
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = FontWeight.SemiBold,
                )
            }
        }
    }
}

@Composable
private fun Status(reference: ReferenceDetailState, onChange: (ReadingStatus) -> Unit) {
    val options = listOf(ReadingStatus.Unread to Res.string.reading_unread, ReadingStatus.Reading to Res.string.reading_reading, ReadingStatus.Read to Res.string.reading_read)
    SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth().padding(horizontal = Dimens.SpaceMd)) {
        options.forEachIndexed { i, (status, label) ->
            SegmentedButton(
                selected = reference.reading == status,
                onClick = { onChange(status) },
                shape = SegmentedButtonDefaults.itemShape(i, options.size),
            ) { Text(stringResource(label)) }
        }
    }
}

@Composable
private fun AbstractSection(abstract: String) {
    var expanded by rememberSaveable { mutableStateOf(false) }
    val long = abstract.length > ABSTRACT_FOLD
    GroupLabel(stringResource(Res.string.section_abstract))
    Group {
        Column(Modifier.padding(Dimens.SpaceMd)) {
            Text(
                if (!long || expanded) abstract else abstract.take(ABSTRACT_FOLD).trimEnd() + "…",
                style = MaterialTheme.typography.bodyMedium,
                color = QuireTheme.colors.fillPrimary,
            )
            if (long) {
                TextButton(onClick = { expanded = !expanded }) {
                    Text(stringResource(if (expanded) Res.string.details_show_less else Res.string.details_show_more))
                }
            }
        }
    }
}

@Composable
private fun InfoSection(reference: ReferenceDetailState) {
    GroupLabel(stringResource(Res.string.section_info))
    Group {
        Column(Modifier.padding(Dimens.SpaceMd), verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            reference.fields.forEach { field ->
                Column {
                    Text(stringResource(field.label), style = MaterialTheme.typography.labelMedium, color = QuireTheme.colors.fillSecondary)
                    Text(field.value, style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillPrimary)
                }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun TagsSection(reference: ReferenceDetailState, state: LibraryState, onEvent: (LibraryEvent) -> Unit) {
    GroupLabel(stringResource(Res.string.section_tags))
    Group {
        Column(Modifier.padding(Dimens.SpaceMd), verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            if (reference.tags.isNotEmpty()) {
                FlowRow(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                    reference.tags.forEach { tag ->
                        InputChip(
                            selected = false,
                            onClick = { onEvent(LibraryEvent.OnTagRemove(tag)) },
                            label = { Text(tag) },
                            trailingIcon = { Icon(painterResource(Res.drawable.ic_close), contentDescription = stringResource(Res.string.details_remove), modifier = Modifier.size(16.dp)) },
                        )
                    }
                }
            }
            AddField(state.tagDraft, stringResource(Res.string.details_add_tag), onChange = { onEvent(LibraryEvent.OnTagDraftChange(it)) }, onAdd = { onEvent(LibraryEvent.OnTagAdd) })
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun ProjectsSection(reference: ReferenceDetailState, state: LibraryState, onEvent: (LibraryEvent) -> Unit) {
    var choosing by remember { mutableStateOf(false) }
    GroupLabel(stringResource(Res.string.section_projects))
    Group {
        Column(Modifier.padding(Dimens.SpaceMd), verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            FlowRow(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                reference.projects.forEachIndexed { i, name ->
                    val path = reference.projectPaths.getOrNull(i)
                    InputChip(
                        selected = false,
                        onClick = { path?.let { onEvent(LibraryEvent.OnProjectRemove(it)) } },
                        label = { Text(name) },
                        trailingIcon = { Icon(painterResource(Res.drawable.ic_close), contentDescription = stringResource(Res.string.details_remove), modifier = Modifier.size(16.dp)) },
                    )
                }
                Box {
                    AssistChip(onClick = { choosing = true }, label = { Text(stringResource(Res.string.details_add_to_project)) })
                    DropdownMenu(expanded = choosing, onDismissRequest = { choosing = false }) {
                        state.projectChoices.filterNot { (path, _) -> path in reference.projectPaths }.forEach { (path, name) ->
                            DropdownMenuItem(
                                text = { Text(name, modifier = Modifier.padding(start = Dimens.SpaceSm * path.count { it == '/' })) },
                                onClick = {
                                    choosing = false
                                    onEvent(LibraryEvent.OnBatchAddToProject(setOf(reference.id), path))
                                },
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun NotesSection(reference: ReferenceDetailState, state: LibraryState, onEvent: (LibraryEvent) -> Unit) {
    GroupLabel(stringResource(Res.string.section_notes))
    Group {
        Column(Modifier.padding(Dimens.SpaceMd), verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm)) {
            if (reference.notes.isEmpty()) {
                Text(stringResource(Res.string.section_empty_notes), style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary)
            }
            reference.notes.forEach { note ->
                Row(verticalAlignment = Alignment.Top) {
                    Column(Modifier.weight(1f)) {
                        Text(note.body, style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillPrimary)
                        Text(note.updated, style = MaterialTheme.typography.labelSmall, color = QuireTheme.colors.fillSecondary)
                    }
                    IconButton(onClick = { onEvent(LibraryEvent.OnNoteDelete(note.id)) }) {
                        Icon(painterResource(Res.drawable.ic_close), contentDescription = stringResource(Res.string.details_remove), tint = QuireTheme.colors.fillSecondary)
                    }
                }
            }
            AddField(state.noteDraft, stringResource(Res.string.details_add_note), onChange = { onEvent(LibraryEvent.OnNoteDraftChange(it)) }, onAdd = { onEvent(LibraryEvent.OnNoteAdd) }, singleLine = false)
        }
    }
}

@Composable
private fun FilesSection(reference: ReferenceDetailState) {
    GroupLabel(stringResource(Res.string.section_attachments))
    Group {
        Column(Modifier.padding(Dimens.SpaceMd), verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            if (reference.pdfFiles.isEmpty()) {
                Text(stringResource(Res.string.section_empty_attachments), style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary)
            }
            reference.pdfFiles.forEach { file ->
                Text(file.substringAfterLast('/'), style = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace), color = QuireTheme.colors.fillPrimary, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}

/** A field with an Add action (the keyboard's Done adds too). */
@Composable
private fun AddField(value: String, label: String, onChange: (String) -> Unit, onAdd: () -> Unit, singleLine: Boolean = true) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        OutlinedTextField(
            value = value,
            onValueChange = onChange,
            label = { Text(label) },
            singleLine = singleLine,
            maxLines = if (singleLine) 1 else 6,
            keyboardOptions = KeyboardOptions(imeAction = if (singleLine) ImeAction.Done else ImeAction.Default),
            keyboardActions = KeyboardActions(onDone = { if (value.isNotBlank()) onAdd() }),
            modifier = Modifier.weight(1f),
        )
        TextButton(onClick = onAdd, enabled = value.isNotBlank()) { Text(stringResource(Res.string.details_add)) }
    }
}

/** Less common actions: update from the source, the publisher's page, trash or restore. */
@Composable
private fun Overflow(reference: ReferenceDetailState, onEvent: (LibraryEvent) -> Unit, onBack: () -> Unit) {
    var open by remember { mutableStateOf(false) }
    Box {
        IconButton(onClick = { open = true }) {
            Icon(painterResource(Res.drawable.ic_more_vert), contentDescription = stringResource(Res.string.details_more), tint = QuireTheme.colors.fillPrimary)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            DropdownMenuItem(text = { Text(stringResource(Res.string.item_refetch)) }, onClick = { open = false; onEvent(LibraryEvent.OnRefetch) })
            DropdownMenuItem(text = { Text(stringResource(Res.string.details_publisher_page)) }, onClick = { open = false; onEvent(LibraryEvent.OnOpenPublisherPage) })
            if (reference.isTrashed) {
                DropdownMenuItem(text = { Text(stringResource(Res.string.item_restore)) }, onClick = { open = false; onEvent(LibraryEvent.OnRestore) })
            } else {
                DropdownMenuItem(
                    text = { Text(stringResource(Res.string.item_delete), color = MaterialTheme.colorScheme.error) },
                    onClick = {
                        open = false
                        onEvent(LibraryEvent.OnDeleteReference(reference.id))
                        onBack()
                    },
                )
            }
        }
    }
}
