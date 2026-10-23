package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.components.cite.CiteButton
import dev.neurokitchen.quire.presentation.components.notice.NoticeBar
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.ExpressiveSprings
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogConfirm
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsGroup
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsItem
import dev.neurokitchen.quire.presentation.mapping.kindName
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.platform.PickKind
import dev.neurokitchen.quire.presentation.platform.rememberFilePicker
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryState
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.library.ReferenceDetailState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.common_link_paper
import dev.neurokitchen.quire.resources.details_add
import dev.neurokitchen.quire.resources.details_add_note
import dev.neurokitchen.quire.resources.details_add_tag
import dev.neurokitchen.quire.resources.details_add_to_project
import dev.neurokitchen.quire.resources.details_attach_from_device
import dev.neurokitchen.quire.resources.details_attach_from_device_sub
import dev.neurokitchen.quire.resources.details_attachments_subtitle
import dev.neurokitchen.quire.resources.details_delete_message
import dev.neurokitchen.quire.resources.details_delete_title
import dev.neurokitchen.quire.resources.details_find_pdf_sub
import dev.neurokitchen.quire.resources.details_more
import dev.neurokitchen.quire.resources.details_note_empty_prompt
import dev.neurokitchen.quire.resources.details_note_new
import dev.neurokitchen.quire.resources.details_note_sheet_subtitle
import dev.neurokitchen.quire.resources.details_projects_none
import dev.neurokitchen.quire.resources.details_projects_sheet_subtitle
import dev.neurokitchen.quire.resources.details_projects_sheet_title
import dev.neurokitchen.quire.resources.details_publisher_page
import dev.neurokitchen.quire.resources.details_read
import dev.neurokitchen.quire.resources.details_remove
import dev.neurokitchen.quire.resources.details_remove_pdfs_sub
import dev.neurokitchen.quire.resources.details_show_less
import dev.neurokitchen.quire.resources.details_show_more
import dev.neurokitchen.quire.resources.details_tags_add_subtitle
import dev.neurokitchen.quire.resources.details_tags_add_title
import dev.neurokitchen.quire.resources.details_tags_empty_hint
import dev.neurokitchen.quire.resources.field_doi
import dev.neurokitchen.quire.resources.field_publication
import dev.neurokitchen.quire.resources.ic_add
import dev.neurokitchen.quire.resources.ic_attach_file
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_content_copy
import dev.neurokitchen.quire.resources.ic_create_new_folder
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_info
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_link
import dev.neurokitchen.quire.resources.ic_more_vert
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import dev.neurokitchen.quire.resources.ic_sell
import dev.neurokitchen.quire.resources.ic_sticky_note_2
import dev.neurokitchen.quire.resources.ic_subject
import dev.neurokitchen.quire.resources.ic_upload
import dev.neurokitchen.quire.resources.item_delete
import dev.neurokitchen.quire.resources.item_get_pdf
import dev.neurokitchen.quire.resources.item_refetch
import dev.neurokitchen.quire.resources.item_remove_pdfs
import dev.neurokitchen.quire.resources.item_restore
import dev.neurokitchen.quire.resources.library_new_collection
import dev.neurokitchen.quire.resources.name_create
import dev.neurokitchen.quire.resources.reading_read
import dev.neurokitchen.quire.resources.reading_reading
import dev.neurokitchen.quire.resources.reading_unread
import dev.neurokitchen.quire.resources.section_abstract
import dev.neurokitchen.quire.resources.section_attachments
import dev.neurokitchen.quire.resources.section_empty_attachments
import dev.neurokitchen.quire.resources.section_info
import dev.neurokitchen.quire.resources.section_notes
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/** An abstract longer than this many characters is folded, with "Show more". */
private const val ABSTRACT_FOLD = 280

/**
 * A paper's details on a phone:
 * - Top bar with Cite, Attachments bottom-sheet trigger, and Overflow menu
 * - Header with Projects pill row, Title, Authors, Venue/Year, and Read / Find PDF action
 * - Semantic Reading Status selector
 * - Horizontal Tags carousel (assigned tags first, then library tags for 1-tap toggling)
 * - Expandable Abstract and compact 2-column Metadata rows
 * - Editorial Notes stream with a bottom-sheet composer
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
    val colors = QuireTheme.colors

    var showProjectsSheet by rememberSaveable { mutableStateOf(false) }
    var showNewProjectDialog by rememberSaveable { mutableStateOf(false) }
    var showAttachmentsSheet by rememberSaveable { mutableStateOf(false) }
    var showAddTagDialog by rememberSaveable { mutableStateOf(false) }
    var showNoteSheet by rememberSaveable { mutableStateOf(false) }
    var showDeleteConfirm by rememberSaveable { mutableStateOf(false) }

    val filePicker = rememberFilePicker { kind, paths ->
        if (kind == PickKind.Pdfs) {
            paths.forEach { path -> onEvent(LibraryEvent.OnAttachPdf(path)) }
        }
    }

    LaunchedEffect(state.pageToOpen) {
        state.pageToOpen?.let { page ->
            runCatching { uriHandler.openUri(page) }
            onEvent(LibraryEvent.OnPageOpened)
        }
    }

    val reference = state.selected

    if (reference != null) {
        if (showProjectsSheet) {
            ProjectsBottomSheet(
                reference = reference,
                projectChoices = state.projectChoices,
                onToggleProject = { path, isAssigned ->
                    if (isAssigned) {
                        onEvent(LibraryEvent.OnProjectRemove(path))
                    } else {
                        onEvent(LibraryEvent.OnBatchAddToProject(setOf(reference.id), path))
                    }
                },
                onNewProject = {
                    showProjectsSheet = false
                    showNewProjectDialog = true
                },
                onDismiss = { showProjectsSheet = false },
            )
        }

        if (showNewProjectDialog) {
            NewArticleProjectDialog(
                onDismiss = { showNewProjectDialog = false },
                onCreate = { name ->
                    showNewProjectDialog = false
                    onEvent(LibraryEvent.OnCreateCollectionFromSelection(name, setOf(reference.id)))
                },
            )
        }

        if (showAttachmentsSheet) {
            AttachmentsBottomSheet(
                reference = reference,
                onOpenPdf = {
                    showAttachmentsSheet = false
                    onOpenReader(reference.id, reference.title)
                },
                onAttachFromDevice = {
                    showAttachmentsSheet = false
                    filePicker.pick(PickKind.Pdfs)
                },
                onFindPdf = {
                    showAttachmentsSheet = false
                    onEvent(LibraryEvent.OnGetPdf)
                },
                onRemovePdfs = {
                    onEvent(LibraryEvent.OnRemovePdfs)
                },
                onDismiss = { showAttachmentsSheet = false },
            )
        }

        if (showAddTagDialog) {
            AddTagDialog(
                draft = state.tagDraft,
                onDraftChange = { onEvent(LibraryEvent.OnTagDraftChange(it)) },
                onConfirm = {
                    onEvent(LibraryEvent.OnTagAdd)
                    showAddTagDialog = false
                },
                onDismiss = { showAddTagDialog = false },
            )
        }

        if (showNoteSheet) {
            NoteComposerSheet(
                draft = state.noteDraft,
                onDraftChange = { onEvent(LibraryEvent.OnNoteDraftChange(it)) },
                onSave = {
                    onEvent(LibraryEvent.OnNoteAdd)
                    showNoteSheet = false
                },
                onDismiss = { showNoteSheet = false },
            )
        }

        if (showDeleteConfirm) {
            DeleteArticleDialog(
                title = reference.title,
                onConfirm = {
                    showDeleteConfirm = false
                    onEvent(LibraryEvent.OnDeleteReference(reference.id))
                    onBack()
                },
                onDismiss = { showDeleteConfirm = false },
            )
        }
    }

    Box(Modifier.fillMaxSize().background(colors.background)) {
        if (reference != null) {
            LazyColumn(
                Modifier.fillMaxSize().hazeSource(chrome.haze),
                contentPadding = mobileContentPadding(extraBottom = 88.dp),
                verticalArrangement = Arrangement.spacedBy(Dimens.SpaceMd),
            ) {
                item {
                    Header(
                        reference = reference,
                        isFetchingPdf = state.isFetchingPdf,
                        onOpenProjects = { showProjectsSheet = true },
                        onRemoveProject = { path -> onEvent(LibraryEvent.OnProjectRemove(path)) },
                        onRead = { onOpenReader(reference.id, reference.title) },
                        onGetPdf = { onEvent(LibraryEvent.OnGetPdf) },
                    )
                }
                item {
                    StatusSelector(reference.reading) { status ->
                        onEvent(LibraryEvent.OnBatchSetReadingStatus(setOf(reference.id), status))
                    }
                }
                item {
                    TagsCarousel(
                        reference = reference,
                        libraryTags = state.tags.map { it.name },
                        onOpenAddTag = { showAddTagDialog = true },
                        onToggleTag = { tag, isAssigned ->
                            if (isAssigned) {
                                onEvent(LibraryEvent.OnTagRemove(tag))
                            } else {
                                onEvent(LibraryEvent.OnTagDraftChange(tag))
                                onEvent(LibraryEvent.OnTagAdd)
                            }
                        },
                    )
                }
                if (reference.abstract.isNotBlank()) {
                    item { AbstractSection(reference.abstract) }
                }
                if (reference.fields.isNotEmpty()) {
                    item { InfoSection(reference) }
                }
                item {
                    NotesSection(
                        reference = reference,
                        onOpenComposer = { showNoteSheet = true },
                        onDeleteNote = { noteId -> onEvent(LibraryEvent.OnNoteDelete(noteId)) },
                    )
                }
            }
        }
        MobileTopBar("", onBack, chrome.haze, actions = {
            if (reference != null) {
                CiteButton(reference.id, reference.citeKey)
                AttachmentsTopBarButton(
                    count = reference.pdfFiles.size,
                    onClick = { showAttachmentsSheet = true },
                )
                Overflow(
                    reference = reference,
                    onEvent = onEvent,
                    onRequestDelete = { showDeleteConfirm = true },
                )
            }
        })
        if (reference != null) {
            ExtendedFloatingActionButton(
                onClick = { showNoteSheet = true },
                icon = {
                    Icon(
                        painter = painterResource(Res.drawable.ic_sticky_note_2),
                        contentDescription = null,
                        modifier = Modifier.size(20.dp),
                    )
                },
                text = {
                    Text(
                        text = stringResource(Res.string.details_note_new),
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = FontWeight.SemiBold,
                    )
                },
                containerColor = colors.accent,
                contentColor = colors.onAccent,
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .navigationBarsPadding()
                    .padding(end = Dimens.SpaceMd, bottom = Dimens.SpaceMd),
            )
        }
        state.notice?.let { notice ->
            NoticeBar(
                notice,
                onDismiss = { onEvent(LibraryEvent.OnDismissNotice) },
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(Dimens.SpaceMd)
                    .padding(bottom = chrome.bottomBar),
            )
        }
    }
}

@Composable
private fun AttachmentsTopBarButton(
    count: Int,
    onClick: () -> Unit,
) {
    val colors = QuireTheme.colors
    Box(contentAlignment = Alignment.Center) {
        IconButton(onClick = onClick) {
            Icon(
                painter = painterResource(Res.drawable.ic_attach_file),
                contentDescription = stringResource(Res.string.section_attachments),
                tint = if (count > 0) colors.accent else colors.fillSecondary,
            )
        }
        if (count > 0) {
            Box(
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(top = 6.dp, end = 6.dp)
                    .size(16.dp)
                    .clip(CircleShape)
                    .background(colors.accent),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = count.toString(),
                    style = MaterialTheme.typography.labelSmall.copy(
                        fontSize = 10.sp,
                        fontWeight = FontWeight.Bold,
                    ),
                    color = colors.onAccent,
                )
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun Header(
    reference: ReferenceDetailState,
    isFetchingPdf: Boolean,
    onOpenProjects: () -> Unit,
    onRemoveProject: (String) -> Unit,
    onRead: () -> Unit,
    onGetPdf: () -> Unit,
) {
    val colors = QuireTheme.colors
    var authorsExpanded by rememberSaveable { mutableStateOf(false) }

    Column(
        Modifier.fillMaxWidth().padding(horizontal = Dimens.SpaceLg),
        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
    ) {
        // Projects & Type/CiteKey Row up top
        FlowRow(
            horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
            modifier = Modifier.fillMaxWidth(),
        ) {
            if (reference.projects.isEmpty()) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    modifier = Modifier
                        .height(30.dp)
                        .clip(RoundedCornerShape(Dimens.RadiusFull))
                        .background(colors.fillQuinary)
                        .clickable(onClick = onOpenProjects)
                        .padding(horizontal = 12.dp),
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_folder),
                        contentDescription = null,
                        tint = colors.sectionProjects,
                        modifier = Modifier.size(15.dp),
                    )
                    Text(
                        text = stringResource(Res.string.details_add_to_project),
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.Medium,
                        color = colors.fillPrimary,
                    )
                }
            } else {
                reference.projects.forEachIndexed { index, projectName ->
                    val path = reference.projectPaths.getOrNull(index)
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                        modifier = Modifier
                            .height(30.dp)
                            .clip(RoundedCornerShape(Dimens.RadiusFull))
                            .background(colors.sectionProjects.copy(alpha = 0.14f))
                            .clickable(onClick = onOpenProjects)
                            .padding(start = 10.dp, end = 6.dp),
                    ) {
                        Icon(
                            painter = painterResource(Res.drawable.ic_folder),
                            contentDescription = null,
                            tint = colors.sectionProjects,
                            modifier = Modifier.size(15.dp),
                        )
                        Text(
                            text = projectName,
                            style = MaterialTheme.typography.labelMedium,
                            fontWeight = FontWeight.SemiBold,
                            color = colors.sectionProjects,
                        )
                        if (path != null) {
                            Box(
                                modifier = Modifier
                                    .size(20.dp)
                                    .clip(CircleShape)
                                    .clickable { onRemoveProject(path) },
                                contentAlignment = Alignment.Center,
                            ) {
                                Icon(
                                    painter = painterResource(Res.drawable.ic_close),
                                    contentDescription = stringResource(Res.string.details_remove),
                                    tint = colors.sectionProjects,
                                    modifier = Modifier.size(13.dp),
                                )
                            }
                        }
                    }
                }
                Box(
                    modifier = Modifier
                        .size(30.dp)
                        .clip(CircleShape)
                        .background(colors.fillQuinary)
                        .clickable(onClick = onOpenProjects),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_add),
                        contentDescription = stringResource(Res.string.details_add_to_project),
                        tint = colors.fillSecondary,
                        modifier = Modifier.size(16.dp),
                    )
                }
            }
        }

        // Title
        Text(
            reference.title,
            style = MaterialTheme.typography.headlineSmall.copy(
                fontFamily = FontFamily.Serif,
                lineHeight = 30.sp,
            ),
            fontWeight = FontWeight.SemiBold,
            color = colors.fillPrimary,
        )

        // Authors (tap to expand if long)
        val authors = reference.authors.joinToString("; ").ifBlank { reference.creator }
        if (authors.isNotBlank()) {
            Text(
                authors,
                style = MaterialTheme.typography.bodyMedium,
                color = colors.fillSecondary,
                maxLines = if (authorsExpanded) Int.MAX_VALUE else 2,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.clickable { authorsExpanded = !authorsExpanded },
            )
        }

        // Year · Venue · Kind
        val venue = reference.fields.firstOrNull { it.label == Res.string.field_publication }?.value.orEmpty()
        val metaLine = listOf(
            reference.year,
            venue.ifBlank { kindName(reference.kind) },
        ).filter { it.isNotBlank() }.joinToString(" · ")
        if (metaLine.isNotBlank()) {
            Text(
                metaLine,
                style = MaterialTheme.typography.bodySmall,
                fontWeight = FontWeight.Medium,
                color = colors.fillSecondary,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }

        Spacer(Modifier.size(Dimens.SpaceMin))

        // Primary Action: Read / Find PDF
        if (!reference.hasPdf) {
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
                    Icon(
                        painter = painterResource(Res.drawable.ic_library_books),
                        contentDescription = null,
                        modifier = Modifier.size(18.dp),
                    )
                    Spacer(Modifier.width(Dimens.SpaceXs))
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
                    colors = ButtonDefaults.buttonColors(
                        containerColor = colors.accent.copy(alpha = 0.16f),
                        contentColor = colors.accent,
                    ),
                    contentPadding = PaddingValues(horizontal = Dimens.SpaceMd),
                ) {
                    if (isFetchingPdf) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(18.dp),
                            color = colors.accent,
                            strokeWidth = 2.dp,
                        )
                    } else {
                        Icon(
                            painter = painterResource(Res.drawable.ic_picture_as_pdf),
                            contentDescription = null,
                            modifier = Modifier.size(18.dp),
                        )
                        Spacer(Modifier.width(Dimens.SpaceXs))
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
                Icon(
                    painter = painterResource(Res.drawable.ic_library_books),
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                )
                Spacer(Modifier.width(Dimens.SpaceXs))
                Text(
                    stringResource(Res.string.details_read),
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = FontWeight.SemiBold,
                )
            }
        }
    }
}

/**
 * Semantic Material 3 Expressive segmented reading status selector.
 */
@Composable
private fun StatusSelector(
    current: ReadingStatus,
    onChange: (ReadingStatus) -> Unit,
) {
    val colors = QuireTheme.colors
    val options = listOf(
        Triple(ReadingStatus.Unread, Res.string.reading_unread, colors.statusUnread),
        Triple(ReadingStatus.Reading, Res.string.reading_reading, colors.statusReading),
        Triple(ReadingStatus.Read, Res.string.reading_read, colors.statusRead),
    )

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = Dimens.SpaceMd),
        horizontalArrangement = Arrangement.spacedBy(3.dp),
    ) {
        options.forEachIndexed { index, (status, labelRes, statusColor) ->
            val selected = current == status
            val shape = when (index) {
                0 -> RoundedCornerShape(topStart = 20.dp, bottomStart = 20.dp, topEnd = 6.dp, bottomEnd = 6.dp)
                options.lastIndex -> RoundedCornerShape(topStart = 6.dp, bottomStart = 6.dp, topEnd = 20.dp, bottomEnd = 20.dp)
                else -> RoundedCornerShape(6.dp)
            }
            val bg = if (selected) statusColor.copy(alpha = 0.18f) else colors.cardBackground
            val fg = if (selected) statusColor else colors.fillSecondary

            Row(
                modifier = Modifier
                    .weight(1f)
                    .height(42.dp)
                    .clip(shape)
                    .background(bg)
                    .clickable { onChange(status) },
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(
                    modifier = Modifier
                        .size(8.dp)
                        .clip(CircleShape)
                        .background(if (selected) statusColor else colors.fillTertiary),
                )
                Spacer(Modifier.width(Dimens.SpaceXs))
                Text(
                    text = stringResource(labelRes),
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                    color = fg,
                )
            }
        }
    }
}

/**
 * Horizontal Tags carousel matching [LibraryFilterRow]:
 * - Leading pill button to add a custom tag
 * - Assigned tags first (filled), followed by other library tags (tonal) for 1-tap toggling
 */
@Composable
private fun TagsCarousel(
    reference: ReferenceDetailState,
    libraryTags: List<String>,
    onOpenAddTag: () -> Unit,
    onToggleTag: (String, Boolean) -> Unit,
) {
    val colors = QuireTheme.colors
    val assigned = reference.tags
    val unassigned = libraryTags.filterNot { it in assigned }
    val allTags = assigned.map { it to true } + unassigned.map { it to false }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = Dimens.SpaceMd),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // Leading '+' Tag action button matching LibraryFilterRow's leading tune button
        Box(
            modifier = Modifier
                .height(36.dp)
                .width(46.dp)
                .clip(RoundedCornerShape(Dimens.RadiusFull))
                .background(colors.fillQuinary)
                .clickable(onClick = onOpenAddTag),
            contentAlignment = Alignment.Center,
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(2.dp),
            ) {
                Icon(
                    painter = painterResource(Res.drawable.ic_sell),
                    contentDescription = stringResource(Res.string.details_add_tag),
                    tint = colors.sectionTags,
                    modifier = Modifier.size(15.dp),
                )
                Icon(
                    painter = painterResource(Res.drawable.ic_add),
                    contentDescription = null,
                    tint = colors.fillPrimary,
                    modifier = Modifier.size(13.dp),
                )
            }
        }

        Spacer(Modifier.width(Dimens.SpaceXs))

        if (allTags.isEmpty()) {
            Box(
                modifier = Modifier
                    .height(36.dp)
                    .clip(RoundedCornerShape(Dimens.RadiusFull))
                    .background(colors.fillQuinary)
                    .clickable(onClick = onOpenAddTag)
                    .padding(horizontal = 16.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = stringResource(Res.string.details_tags_empty_hint),
                    style = MaterialTheme.typography.labelMedium,
                    color = colors.fillSecondary,
                )
            }
        } else {
            LazyRow(
                horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                items(allTags, key = { (tag, _) -> tag }) { (tag, isAssigned) ->
                    Row(
                        modifier = Modifier
                            .height(36.dp)
                            .clip(RoundedCornerShape(Dimens.RadiusFull))
                            .background(if (isAssigned) colors.accent else colors.fillQuinary)
                            .clickable { onToggleTag(tag, isAssigned) }
                            .padding(horizontal = 14.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        Text(
                            text = "#$tag",
                            style = MaterialTheme.typography.labelMedium.copy(
                                fontWeight = if (isAssigned) FontWeight.Bold else FontWeight.Medium,
                                fontSize = 13.sp,
                            ),
                            color = if (isAssigned) colors.onAccent else colors.fillPrimary,
                        )
                        if (isAssigned) {
                            Icon(
                                painter = painterResource(Res.drawable.ic_close),
                                contentDescription = stringResource(Res.string.details_remove),
                                tint = colors.onAccent.copy(alpha = 0.85f),
                                modifier = Modifier.size(14.dp),
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun SectionHeader(
    icon: DrawableResource,
    title: String,
    tint: Color,
    count: Int? = null,
    trailing: (@Composable () -> Unit)? = null,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = Dimens.SpaceLg, end = Dimens.SpaceMd, bottom = Dimens.SpaceXs),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            painter = painterResource(icon),
            contentDescription = null,
            tint = tint,
            modifier = Modifier.size(18.dp),
        )
        Spacer(Modifier.width(Dimens.SpaceXs))
        Text(
            text = title,
            style = MaterialTheme.typography.titleSmall,
            fontWeight = FontWeight.Bold,
            color = tint,
        )
        if (count != null && count > 0) {
            Spacer(Modifier.width(Dimens.SpaceXs))
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(Dimens.RadiusFull))
                    .background(tint.copy(alpha = 0.15f))
                    .padding(horizontal = 8.dp, vertical = 2.dp),
            ) {
                Text(
                    text = count.toString(),
                    style = MaterialTheme.typography.labelSmall,
                    fontWeight = FontWeight.Bold,
                    color = tint,
                )
            }
        }
        Spacer(Modifier.weight(1f))
        trailing?.invoke()
    }
}

@Composable
private fun DetailsCard(content: @Composable () -> Unit) {
    Box(
        Modifier
            .fillMaxWidth()
            .padding(horizontal = Dimens.SpaceMd)
            .clip(RoundedCornerShape(Dimens.RadiusLarge))
            .background(QuireTheme.colors.cardBackground),
    ) {
        content()
    }
}

@Composable
private fun AbstractSection(abstract: String) {
    val colors = QuireTheme.colors
    var expanded by rememberSaveable { mutableStateOf(false) }
    val long = abstract.length > ABSTRACT_FOLD

    Column {
        SectionHeader(
            icon = Res.drawable.ic_subject,
            title = stringResource(Res.string.section_abstract),
            tint = colors.sectionAbstract,
        )
        DetailsCard {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable(enabled = long) { expanded = !expanded }
                    .animateContentSize(ExpressiveSprings.smooth())
                    .padding(Dimens.SpaceMd),
                verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
            ) {
                Text(
                    text = abstract,
                    style = MaterialTheme.typography.bodyMedium.copy(lineHeight = 22.sp),
                    color = colors.fillPrimary,
                    maxLines = if (!long || expanded) Int.MAX_VALUE else 5,
                    overflow = TextOverflow.Ellipsis,
                )
                if (long) {
                    Text(
                        text = stringResource(if (expanded) Res.string.details_show_less else Res.string.details_show_more),
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.SemiBold,
                        color = colors.accent,
                    )
                }
            }
        }
    }
}

@Composable
private fun InfoSection(reference: ReferenceDetailState) {
    val colors = QuireTheme.colors
    val clipboard = LocalClipboardManager.current

    Column {
        SectionHeader(
            icon = Res.drawable.ic_info,
            title = stringResource(Res.string.section_info),
            tint = colors.sectionInfo,
        )
        DetailsCard {
            Column(
                modifier = Modifier.padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceSm),
                verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
            ) {
                reference.fields.forEachIndexed { index, field ->
                    val isDoi = field.label == Res.string.field_doi
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .then(
                                if (isDoi) {
                                    Modifier
                                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                        .clickable { clipboard.setText(AnnotatedString(field.value)) }
                                } else {
                                    Modifier
                                },
                            )
                            .padding(vertical = 4.dp),
                        horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
                        verticalAlignment = Alignment.Top,
                    ) {
                        Text(
                            text = stringResource(field.label),
                            style = MaterialTheme.typography.bodySmall,
                            color = colors.fillSecondary,
                            modifier = Modifier.width(96.dp),
                        )
                        Text(
                            text = field.value,
                            style = MaterialTheme.typography.bodyMedium.copy(
                                fontFamily = if (isDoi) FontFamily.Monospace else FontFamily.Default,
                            ),
                            fontWeight = FontWeight.Medium,
                            color = if (isDoi) colors.accent else colors.fillPrimary,
                            modifier = Modifier.weight(1f),
                        )
                        if (isDoi) {
                            Icon(
                                painter = painterResource(Res.drawable.ic_content_copy),
                                contentDescription = null,
                                tint = colors.accent,
                                modifier = Modifier.size(16.dp),
                            )
                        }
                    }
                    if (index < reference.fields.lastIndex) {
                        HorizontalDivider(color = colors.border.copy(alpha = 0.35f))
                    }
                }
            }
        }
    }
}

/**
 * Editorial Notes stream directly on the page (no enclosing settings box or permanent text field).
 */
@Composable
private fun NotesSection(
    reference: ReferenceDetailState,
    onOpenComposer: () -> Unit,
    onDeleteNote: (String) -> Unit,
) {
    val colors = QuireTheme.colors
    val wikilinkRegex = remember { Regex("\\[\\[([a-zA-Z0-9_:-]+)\\]\\]") }

    Column {
        SectionHeader(
            icon = Res.drawable.ic_sticky_note_2,
            title = stringResource(Res.string.section_notes),
            tint = colors.sectionNotes,
            count = reference.notes.size,
        )

        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = Dimens.SpaceMd),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
        ) {
            if (reference.notes.isEmpty()) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(IntrinsicSize.Min)
                        .clip(RoundedCornerShape(Dimens.RadiusBase))
                        .background(colors.surfaceContainerLow)
                        .clickable(onClick = onOpenComposer)
                        .padding(vertical = 14.dp, horizontal = Dimens.SpaceMd),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
                ) {
                    Box(
                        modifier = Modifier
                            .width(4.dp)
                            .fillMaxHeight()
                            .clip(RoundedCornerShape(2.dp))
                            .background(colors.sectionNotes.copy(alpha = 0.6f)),
                    )
                    Text(
                        text = stringResource(Res.string.details_note_empty_prompt),
                        style = MaterialTheme.typography.bodyMedium,
                        color = colors.fillSecondary,
                        modifier = Modifier.weight(1f),
                    )
                }
            } else {
                reference.notes.forEach { note ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(IntrinsicSize.Min)
                            .clip(RoundedCornerShape(Dimens.RadiusBase))
                            .background(colors.surfaceContainerLow)
                            .padding(start = Dimens.SpaceSm, end = Dimens.SpaceXs, top = Dimens.SpaceSm, bottom = Dimens.SpaceSm),
                        verticalAlignment = Alignment.Top,
                    ) {
                        Box(
                            modifier = Modifier
                                .width(4.dp)
                                .fillMaxHeight()
                                .clip(RoundedCornerShape(2.dp))
                                .background(colors.sectionNotes),
                        )
                        Spacer(Modifier.width(Dimens.SpaceSm))
                        Column(
                            modifier = Modifier.weight(1f),
                            verticalArrangement = Arrangement.spacedBy(4.dp),
                        ) {
                            val annotated = remember(note.body, colors.accent) {
                                buildAnnotatedString {
                                    var lastIdx = 0
                                    wikilinkRegex.findAll(note.body).forEach { match ->
                                        append(note.body.substring(lastIdx, match.range.first))
                                        pushStyle(SpanStyle(color = colors.accent, fontWeight = FontWeight.Bold))
                                        append(match.value)
                                        pop()
                                        lastIdx = match.range.last + 1
                                    }
                                    if (lastIdx < note.body.length) {
                                        append(note.body.substring(lastIdx))
                                    }
                                }
                            }
                            Text(
                                text = annotated,
                                style = MaterialTheme.typography.bodyMedium.copy(lineHeight = 21.sp),
                                color = colors.fillPrimary,
                            )
                            Text(
                                text = note.updated,
                                style = MaterialTheme.typography.labelSmall,
                                color = colors.fillSecondary,
                            )
                        }
                        IconButton(
                            onClick = { onDeleteNote(note.id) },
                            modifier = Modifier.size(32.dp),
                        ) {
                            Icon(
                                painter = painterResource(Res.drawable.ic_close),
                                contentDescription = stringResource(Res.string.details_remove),
                                tint = colors.fillSecondary,
                                modifier = Modifier.size(16.dp),
                            )
                        }
                    }
                }
            }
        }
    }
}

/** Less common actions: update from the source, the publisher's page, trash or restore. */
@Composable
private fun Overflow(
    reference: ReferenceDetailState,
    onEvent: (LibraryEvent) -> Unit,
    onRequestDelete: () -> Unit,
) {
    var open by remember { mutableStateOf(false) }
    Box {
        IconButton(onClick = { open = true }) {
            Icon(
                painterResource(Res.drawable.ic_more_vert),
                contentDescription = stringResource(Res.string.details_more),
                tint = QuireTheme.colors.fillPrimary,
            )
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            DropdownMenuItem(
                text = { Text(stringResource(Res.string.item_refetch)) },
                onClick = {
                    open = false
                    onEvent(LibraryEvent.OnRefetch)
                },
            )
            DropdownMenuItem(
                text = { Text(stringResource(Res.string.details_publisher_page)) },
                onClick = {
                    open = false
                    onEvent(LibraryEvent.OnOpenPublisherPage)
                },
            )
            if (reference.isTrashed) {
                DropdownMenuItem(
                    text = { Text(stringResource(Res.string.item_restore)) },
                    onClick = {
                        open = false
                        onEvent(LibraryEvent.OnRestore)
                    },
                )
            } else {
                DropdownMenuItem(
                    text = { Text(stringResource(Res.string.item_delete), color = MaterialTheme.colorScheme.error) },
                    onClick = {
                        open = false
                        onRequestDelete()
                    },
                )
            }
        }
    }
}
