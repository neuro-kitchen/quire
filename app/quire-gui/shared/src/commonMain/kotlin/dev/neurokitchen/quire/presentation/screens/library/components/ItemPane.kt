package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import dev.neurokitchen.quire.presentation.designsystem.components.CompactTextField
import dev.neurokitchen.quire.presentation.platform.pickFile
import dev.neurokitchen.quire.presentation.screens.library.EditDraft
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.resources.field_date
import dev.neurokitchen.quire.resources.field_doi
import dev.neurokitchen.quire.resources.field_issue
import dev.neurokitchen.quire.resources.field_pages
import dev.neurokitchen.quire.resources.field_publication
import dev.neurokitchen.quire.resources.field_volume
import dev.neurokitchen.quire.resources.item_add_note
import dev.neurokitchen.quire.resources.item_add_tag
import dev.neurokitchen.quire.resources.item_attach_pdf
import dev.neurokitchen.quire.resources.item_cancel
import dev.neurokitchen.quire.resources.item_collapse
import dev.neurokitchen.quire.resources.item_delete
import dev.neurokitchen.quire.resources.item_edit
import dev.neurokitchen.quire.resources.item_get_pdf
import dev.neurokitchen.quire.resources.item_pick_pdf
import dev.neurokitchen.quire.resources.item_publisher_page
import dev.neurokitchen.quire.resources.item_refetch
import dev.neurokitchen.quire.resources.item_remove
import dev.neurokitchen.quire.resources.item_remove_pdfs
import dev.neurokitchen.quire.resources.item_restore
import dev.neurokitchen.quire.resources.item_save
import dev.neurokitchen.quire.resources.section_abstract
import dev.neurokitchen.quire.resources.status_read
import dev.neurokitchen.quire.resources.status_reading
import dev.neurokitchen.quire.resources.status_unread
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInParent
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mapping.kindName
import dev.neurokitchen.quire.presentation.screens.library.ItemSection
import dev.neurokitchen.quire.presentation.screens.library.ReferenceDetailState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.attachment_pdf
import dev.neurokitchen.quire.resources.field_author
import dev.neurokitchen.quire.resources.field_cite_key
import dev.neurokitchen.quire.resources.field_item_type
import dev.neurokitchen.quire.resources.field_title
import dev.neurokitchen.quire.resources.ic_add
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_attach_file
import dev.neurokitchen.quire.resources.ic_create_new_folder
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_expand_more
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_info
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_link
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import dev.neurokitchen.quire.resources.ic_sell
import dev.neurokitchen.quire.resources.ic_sticky_note_2
import dev.neurokitchen.quire.resources.ic_subject
import dev.neurokitchen.quire.resources.library_item_count
import dev.neurokitchen.quire.resources.reader_open
import dev.neurokitchen.quire.resources.section_abstract
import dev.neurokitchen.quire.resources.section_attachments
import dev.neurokitchen.quire.resources.section_empty_abstract
import dev.neurokitchen.quire.resources.section_empty_attachments
import dev.neurokitchen.quire.resources.section_empty_notes
import dev.neurokitchen.quire.resources.section_empty_projects
import dev.neurokitchen.quire.resources.section_empty_related
import dev.neurokitchen.quire.resources.section_empty_tags
import dev.neurokitchen.quire.resources.section_info
import dev.neurokitchen.quire.resources.section_notes
import dev.neurokitchen.quire.resources.section_projects
import dev.neurokitchen.quire.resources.section_related
import dev.neurokitchen.quire.resources.section_tags
import kotlinx.coroutines.launch
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

/**
 * Quire item pane:
 * - When no reference is selected: displays the Library Dashboard (reading stats, quick actions, shortcuts).
 * - When a reference is selected: title header card, quick actions, and collapsible sections.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ItemPane(
    reference: ReferenceDetailState?,
    itemCount: Int,
    collapsed: Set<ItemSection>,
    onSectionToggle: (ItemSection) -> Unit,
    modifier: Modifier = Modifier,
    /** Opens the selected reference in the reader. */
    onOpen: () -> Unit = {},
    onDelete: () -> Unit = {},
    onAddReference: () -> Unit = {},
    onCreateProject: () -> Unit = {},
    onCollapse: (() -> Unit)? = null,
    /** The Info section being edited, and what is typed in the Tags and Notes inputs. */
    editing: EditDraft? = null,
    tagDraft: String = "",
    noteDraft: String = "",
    onEvent: (LibraryEvent) -> Unit = {},
) {
    val colors = QuireTheme.colors
    if (reference == null) {
        LibraryDashboard(itemCount, onAddReference, onCreateProject, onCollapse, modifier)
        return
    }
    val scroll = rememberScrollState()
    val scope = rememberCoroutineScope()
    val offsets = remember(reference.id) { mutableMapOf<ItemSection, Int>() }
    Row(modifier.fillMaxSize().background(colors.background)) {
        Column(Modifier.weight(1f).fillMaxHeight().verticalScroll(scroll)) {
            // Expressive Header
            Column(Modifier.padding(start = Dimens.SpaceMd, end = Dimens.SpaceMd, top = Dimens.SpaceMd, bottom = Dimens.SpaceXs)) {
                Text(
                    reference.title,
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.fillPrimary,
                )
                Spacer(Modifier.height(Dimens.SpaceMin))
                Text(
                    reference.creator + if (reference.year.isNotEmpty()) " (${reference.year})" else "",
                    style = MaterialTheme.typography.bodyMedium,
                    color = colors.fillSecondary,
                )
                Spacer(Modifier.height(Dimens.SpaceSm))

                // Action pills; the actions wrap to their own line when the pane is narrow
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
                    verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Button(
                        onClick = onOpen,
                        shape = RoundedCornerShape(Dimens.RadiusBase),
                        colors = ButtonDefaults.buttonColors(containerColor = colors.accent),
                        contentPadding = PaddingValues(horizontal = Dimens.SpaceSm, vertical = 4.dp),
                        modifier = Modifier.height(30.dp),
                    ) {
                        Icon(painterResource(Res.drawable.ic_picture_as_pdf), null, modifier = Modifier.size(14.dp))
                        Spacer(Modifier.width(Dimens.SpaceMin))
                        Text(stringResource(Res.string.reader_open), style = MaterialTheme.typography.labelSmall)
                    }

                    // Quick Look preview button
                    Button(
                        onClick = { onEvent(LibraryEvent.OnToggleQuickLook(true)) },
                        shape = RoundedCornerShape(Dimens.RadiusBase),
                        colors = ButtonDefaults.buttonColors(containerColor = colors.sidepane),
                        contentPadding = PaddingValues(horizontal = Dimens.SpaceSm, vertical = 4.dp),
                        modifier = Modifier
                            .height(30.dp)
                            .border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusBase)),
                    ) {
                        Icon(painterResource(Res.drawable.ic_description), null, tint = colors.fillPrimary, modifier = Modifier.size(13.dp))
                        Spacer(Modifier.width(Dimens.SpaceMin))
                        Text("Quick Look", style = MaterialTheme.typography.labelSmall, color = colors.fillPrimary)
                    }

                    // Reading status badge
                    val (statusBg, statusFg, statusText) = when (reference.reading) {
                        ReadingStatus.Read -> Triple(colors.statusRead.copy(alpha = 0.15f), colors.statusRead, stringResource(Res.string.status_read))
                        ReadingStatus.Reading -> Triple(colors.statusReading.copy(alpha = 0.15f), colors.statusReading, stringResource(Res.string.status_reading))
                        ReadingStatus.Unread -> Triple(colors.fillQuarternary, colors.fillSecondary, stringResource(Res.string.status_unread))
                    }
                    Box(
                        modifier = Modifier
                            .height(30.dp)
                            .clip(RoundedCornerShape(Dimens.RadiusFull))
                            .background(statusBg)
                            .padding(horizontal = 10.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(statusText, style = MaterialTheme.typography.labelSmall, color = statusFg, fontWeight = FontWeight.Medium)
                    }

                    // CiteKey badge
                    Box(
                        modifier = Modifier
                            .height(30.dp)
                            .clip(RoundedCornerShape(Dimens.RadiusSmall))
                            .background(colors.fillQuarternary)
                            .padding(horizontal = 8.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            reference.citeKey,
                            style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace),
                            color = colors.fillPrimary,
                        )
                    }

                    Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.height(30.dp)) {
                        if (reference.isTrashed) {
                            Action(Res.string.item_restore, { onEvent(LibraryEvent.OnRestore) })
                        } else {
                            Action(Res.string.item_refetch, { onEvent(LibraryEvent.OnRefetch) })
                            IconButton(onClick = onDelete, modifier = Modifier.size(30.dp)) {
                                Icon(painterResource(Res.drawable.ic_delete), stringResource(Res.string.item_delete), tint = colors.fillSecondary, modifier = Modifier.size(16.dp))
                            }
                        }

                        if (onCollapse != null) {
                            IconButton(onClick = onCollapse, modifier = Modifier.size(30.dp)) {
                                Icon(painterResource(Res.drawable.ic_close), stringResource(Res.string.item_collapse), tint = colors.fillSecondary, modifier = Modifier.size(16.dp))
                            }
                        }
                    }
                }
            }

            HorizontalDivider(color = colors.fillQuarternary)
            ItemSection.entries.forEach { section ->
                Column(Modifier.onGloballyPositioned { offsets[section] = it.positionInParent().y.toInt() }) {
                    Section(section, isCollapsed = section in collapsed, onToggle = { onSectionToggle(section) }) {
                        SectionBody(section, reference, editing, tagDraft, noteDraft, onOpen, onEvent)
                    }
                    HorizontalDivider(color = colors.fillQuarternary)
                }
            }
        }
        VerticalDivider(color = colors.paneDivider)
        SideNav(onSectionClick = { section ->
            offsets[section]?.let { scope.launch { scroll.animateScrollTo(it) } }
        })
    }
}

@Composable
private fun LibraryDashboard(
    itemCount: Int,
    onAddReference: () -> Unit,
    onCreateProject: () -> Unit,
    onCollapse: (() -> Unit)? = null,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(colors.sidepane)
            .padding(Dimens.SpaceMd)
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
    ) {
        // Header
        Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(
                painterResource(Res.drawable.ic_library_books),
                contentDescription = null,
                tint = colors.accent,
                modifier = Modifier.size(20.dp),
            )
            Spacer(Modifier.width(8.dp))
            Text(
                "Library Overview",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = colors.fillPrimary,
            )
            Spacer(Modifier.weight(1f))
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(Dimens.RadiusFull))
                    .background(colors.accent.copy(alpha = 0.15f))
                    .padding(horizontal = 8.dp, vertical = 2.dp),
            ) {
                Text(
                    text = "$itemCount items",
                    style = MaterialTheme.typography.labelSmall,
                    color = colors.accent,
                    fontWeight = FontWeight.SemiBold,
                )
            }
            if (onCollapse != null) {
                Spacer(Modifier.width(6.dp))
                IconButton(
                    onClick = onCollapse,
                    modifier = Modifier.size(26.dp),
                ) {
                    Icon(painterResource(Res.drawable.ic_close), "Minimize Inspector (Ctrl+I)", tint = colors.fillSecondary, modifier = Modifier.size(14.dp))
                }
            }
        }

        // Quick Actions Card
        Card(
            shape = RoundedCornerShape(Dimens.RadiusBase),
            colors = CardDefaults.cardColors(containerColor = colors.background),
            modifier = Modifier.fillMaxWidth().border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusBase)),
        ) {
            Column(Modifier.padding(Dimens.SpaceSm)) {
                Text(
                    "Quick Actions",
                    style = MaterialTheme.typography.labelMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.fillPrimary,
                )
                Spacer(Modifier.height(Dimens.SpaceXs))
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                        .clickable(onClick = onAddReference)
                        .padding(vertical = 6.dp, horizontal = 4.dp),
                ) {
                    Icon(painterResource(Res.drawable.ic_add), null, tint = colors.accent, modifier = Modifier.size(16.dp))
                    Spacer(Modifier.width(8.dp))
                    Text("Add Reference (DOI, arXiv, BibTeX)", style = MaterialTheme.typography.bodySmall, color = colors.fillPrimary)
                }
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                        .clickable(onClick = onCreateProject)
                        .padding(vertical = 6.dp, horizontal = 4.dp),
                ) {
                    Icon(painterResource(Res.drawable.ic_create_new_folder), null, tint = colors.accent, modifier = Modifier.size(16.dp))
                    Spacer(Modifier.width(8.dp))
                    Text("New Collection / Project", style = MaterialTheme.typography.bodySmall, color = colors.fillPrimary)
                }
            }
        }

        // Keyboard Shortcuts Card
        Card(
            shape = RoundedCornerShape(Dimens.RadiusBase),
            colors = CardDefaults.cardColors(containerColor = colors.background),
            modifier = Modifier.fillMaxWidth().border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusBase)),
        ) {
            Column(Modifier.padding(Dimens.SpaceSm), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(
                    "Keyboard Shortcuts",
                    style = MaterialTheme.typography.labelMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.fillPrimary,
                )
                ShortcutRow("Ctrl + B", "Toggle Left Sidebar")
                ShortcutRow("Ctrl + F / Ctrl + S", "Search Library")
                ShortcutRow("Ctrl + N", "Add Reference Modal")
                ShortcutRow("Ctrl + R", "Reload Library")
                ShortcutRow("Ctrl + ,", "Settings")
            }
        }

        // Reader Hint Card
        Card(
            shape = RoundedCornerShape(Dimens.RadiusBase),
            colors = CardDefaults.cardColors(containerColor = colors.background),
            modifier = Modifier.fillMaxWidth().border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusBase)),
        ) {
            Column(Modifier.padding(Dimens.SpaceSm)) {
                Text(
                    "Deep Reading Experience",
                    style = MaterialTheme.typography.labelMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.fillPrimary,
                )
                Spacer(Modifier.height(4.dp))
                Text(
                    "Double-click any paper to open the Reader. Supports PDF view & distraction-free text mode with 6 color themes, font selection, and floating highlight palette.",
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.fillSecondary,
                )
            }
        }
    }
}

@Composable
private fun ShortcutRow(shortcut: String, description: String) {
    val colors = QuireTheme.colors
    Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
        Box(
            modifier = Modifier
                .clip(RoundedCornerShape(4.dp))
                .background(colors.fillQuarternary)
                .padding(horizontal = 6.dp, vertical = 2.dp),
        ) {
            Text(shortcut, style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace, fontSize = 10.sp), color = colors.fillPrimary)
        }
        Spacer(Modifier.width(8.dp))
        Text(description, style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary)
    }
}

private val ItemSection.title: StringResource
    get() = when (this) {
        ItemSection.Info -> Res.string.section_info
        ItemSection.Abstract -> Res.string.section_abstract
        ItemSection.Attachments -> Res.string.section_attachments
        ItemSection.Notes -> Res.string.section_notes
        ItemSection.Projects -> Res.string.section_projects
        ItemSection.Tags -> Res.string.section_tags
        ItemSection.Related -> Res.string.section_related
    }

private val ItemSection.icon: DrawableResource
    get() = when (this) {
        ItemSection.Info -> Res.drawable.ic_info
        ItemSection.Abstract -> Res.drawable.ic_subject
        ItemSection.Attachments -> Res.drawable.ic_attach_file
        ItemSection.Notes -> Res.drawable.ic_sticky_note_2
        ItemSection.Projects -> Res.drawable.ic_folder
        ItemSection.Tags -> Res.drawable.ic_sell
        ItemSection.Related -> Res.drawable.ic_link
    }

@Composable
private fun ItemSection.color(): Color = QuireTheme.colors.let {
    when (this) {
        ItemSection.Info -> it.sectionInfo
        ItemSection.Abstract -> it.sectionAbstract
        ItemSection.Attachments -> it.sectionAttachments
        ItemSection.Notes -> it.sectionNotes
        ItemSection.Projects -> it.sectionProjects
        ItemSection.Tags -> it.sectionTags
        ItemSection.Related -> it.sectionRelated
    }
}

@Composable
private fun Section(
    section: ItemSection,
    isCollapsed: Boolean,
    onToggle: () -> Unit,
    content: @Composable () -> Unit,
) {
    val colors = QuireTheme.colors
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.fillMaxWidth().clickable(onClick = onToggle).padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceXs),
    ) {
        Icon(painterResource(section.icon), null, tint = section.color(), modifier = Modifier.size(Dimens.Icon))
        Text(
            stringResource(section.title),
            style = MaterialTheme.typography.titleMedium,
            color = colors.fillPrimary,
            modifier = Modifier.padding(start = Dimens.SpaceXs).weight(1f),
        )
        Icon(
            painterResource(Res.drawable.ic_expand_more), null, tint = colors.fillSecondary,
            modifier = Modifier.size(Dimens.Icon).rotate(if (isCollapsed) -90f else 0f),
        )
    }
    if (!isCollapsed) {
        Box(Modifier.fillMaxWidth().padding(start = Dimens.SpaceMd, end = Dimens.SpaceMd, bottom = Dimens.SpaceSm)) { content() }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun SectionBody(
    section: ItemSection,
    reference: ReferenceDetailState,
    editing: EditDraft?,
    tagDraft: String,
    noteDraft: String,
    onOpen: () -> Unit,
    onEvent: (LibraryEvent) -> Unit,
) {
    val colors = QuireTheme.colors
    when (section) {
        ItemSection.Info -> Column {
            if (reference.issues.isNotEmpty()) {
                reference.issues.forEach { Text("• $it", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error) }
                Spacer(Modifier.height(Dimens.SpaceXs))
            }
            if (editing != null) {
                EditFields(editing) { onEvent(LibraryEvent.OnEditChange(it)) }
                Row(horizontalArrangement = Arrangement.End, modifier = Modifier.fillMaxWidth()) {
                    Action(Res.string.item_cancel, { onEvent(LibraryEvent.OnEditCancel) })
                    Action(Res.string.item_save, { onEvent(LibraryEvent.OnEditSave) })
                }
            } else {
                Field(stringResource(Res.string.field_item_type), kindName(reference.kind))
                Field(stringResource(Res.string.field_title), reference.title)
                reference.authors.forEach { Field(stringResource(Res.string.field_author), it) }
                reference.fields.forEach { Field(stringResource(it.label), it.value) }
                Field(stringResource(Res.string.field_cite_key), reference.citeKey)
                if (!reference.isTrashed) {
                    Action(Res.string.item_edit, { onEvent(LibraryEvent.OnEditStart) })
                }
            }
        }
        ItemSection.Abstract -> BodyText(reference.abstract, Res.string.section_empty_abstract)
        ItemSection.Attachments -> Column {
            if (reference.pdfFiles.isEmpty()) BodyText("", Res.string.section_empty_attachments)
            reference.pdfFiles.forEach { path ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.fillMaxWidth().height(Dimens.RowHeight).clickable(onClick = onOpen),
                ) {
                    Icon(painterResource(Res.drawable.ic_picture_as_pdf), null, tint = colors.accentRed, modifier = Modifier.size(Dimens.Icon))
                    Text(
                        path.substringAfterLast('/').substringAfterLast('\\'),
                        style = MaterialTheme.typography.bodySmall,
                        maxLines = 1,
                        modifier = Modifier.padding(start = Dimens.SpaceXs),
                    )
                }
            }
            val pickTitle = stringResource(Res.string.item_pick_pdf)
            FlowRow(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                if (reference.pdfFiles.isEmpty()) {
                    Action(Res.string.item_get_pdf, { onEvent(LibraryEvent.OnGetPdf) })
                    Action(Res.string.item_publisher_page, { onEvent(LibraryEvent.OnOpenPublisherPage) })
                }
                Action(Res.string.item_attach_pdf) { pickFile(pickTitle, listOf("pdf"))?.let { onEvent(LibraryEvent.OnAttachPdf(it)) } }
                if (reference.pdfFiles.isNotEmpty()) {
                    Action(Res.string.item_remove_pdfs, { onEvent(LibraryEvent.OnRemovePdfs) })
                }
            }
        }
        ItemSection.Notes -> Column(verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            val wikilinkRegex = remember { Regex("\\[\\[([a-zA-Z0-9_:-]+)\\]\\]") }
            if (reference.notes.isEmpty()) BodyText("", Res.string.section_empty_notes)
            reference.notes.forEach { note ->
                Row(verticalAlignment = Alignment.Top) {
                    Icon(painterResource(Res.drawable.ic_sticky_note_2), null, tint = colors.sectionNotes, modifier = Modifier.size(16.dp))
                    Column(Modifier.weight(1f).padding(start = Dimens.SpaceXs)) {
                        val annotated = remember(note.body, colors.accent) {
                            androidx.compose.ui.text.buildAnnotatedString {
                                var lastIdx = 0
                                wikilinkRegex.findAll(note.body).forEach { match ->
                                    append(note.body.substring(lastIdx, match.range.first))
                                    pushStyle(androidx.compose.ui.text.SpanStyle(color = colors.accent, fontWeight = FontWeight.Bold))
                                    append(match.value)
                                    pop()
                                    lastIdx = match.range.last + 1
                                }
                                if (lastIdx < note.body.length) {
                                    append(note.body.substring(lastIdx))
                                }
                            }
                        }
                        Text(annotated, style = MaterialTheme.typography.bodySmall, color = colors.fillPrimary)
                        Text(note.updated, style = MaterialTheme.typography.labelSmall, color = colors.fillSecondary)
                    }
                    RemoveButton { onEvent(LibraryEvent.OnNoteDelete(note.id)) }
                }
            }
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth().padding(vertical = 2.dp),
            ) {
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                        .background(colors.accent.copy(alpha = 0.12f))
                        .clickable {
                            val cur = noteDraft
                            val next = if (cur.isEmpty() || cur.endsWith(" ")) cur + "[[key]]" else "$cur [[key]]"
                            onEvent(LibraryEvent.OnNoteDraftChange(next))
                        }
                        .padding(horizontal = 6.dp, vertical = 2.dp),
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(painterResource(Res.drawable.ic_link), null, tint = colors.accent, modifier = Modifier.size(11.dp))
                        Spacer(Modifier.width(4.dp))
                        Text("+ Link [[Paper]]", style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp, fontWeight = FontWeight.SemiBold), color = colors.accent)
                    }
                }
            }
            InputRow(noteDraft, stringResource(Res.string.item_add_note) + " ([[key]] to link)", { onEvent(LibraryEvent.OnNoteDraftChange(it)) }) {
                onEvent(LibraryEvent.OnNoteAdd)
            }

            // Backlinks & Mentions Section
            if (reference.citeKey.isNotBlank()) {
                Spacer(Modifier.height(8.dp))
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                        .background(colors.fillQuarternary.copy(alpha = 0.5f))
                        .padding(horizontal = 8.dp, vertical = 4.dp),
                ) {
                    Icon(painterResource(Res.drawable.ic_link), null, tint = colors.accent, modifier = Modifier.size(13.dp))
                    Spacer(Modifier.width(6.dp))
                    Text(
                        "BACKLINKS TO [[@${reference.citeKey}]]",
                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp, fontWeight = FontWeight.Bold),
                        color = colors.accent,
                    )
                }
            }
        }
        ItemSection.Projects -> Column {
            if (reference.projects.isEmpty()) BodyText("", Res.string.section_empty_projects)
            reference.projects.forEachIndexed { i, name ->
                Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.height(Dimens.RowHeight)) {
                    Icon(painterResource(Res.drawable.ic_folder), null, tint = colors.accent, modifier = Modifier.size(Dimens.Icon))
                    Text(name, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.padding(start = Dimens.SpaceXs).weight(1f))
                    reference.projectPaths.getOrNull(i)?.let { path -> RemoveButton { onEvent(LibraryEvent.OnProjectRemove(path)) } }
                }
            }
        }
        ItemSection.Tags -> Column(verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            if (reference.tags.isEmpty()) BodyText("", Res.string.section_empty_tags)
            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                reference.tags.forEach { tag ->
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier
                            .clip(RoundedCornerShape(Dimens.RadiusFull))
                            .background(colors.accent.copy(alpha = 0.12f))
                            .padding(start = 8.dp),
                    ) {
                        Text("#$tag", style = MaterialTheme.typography.labelSmall, color = colors.accent, fontWeight = FontWeight.Medium)
                        RemoveButton { onEvent(LibraryEvent.OnTagRemove(tag)) }
                    }
                }
            }
            InputRow(tagDraft, stringResource(Res.string.item_add_tag), { onEvent(LibraryEvent.OnTagDraftChange(it)) }) {
                onEvent(LibraryEvent.OnTagAdd)
            }
        }
        ItemSection.Related -> Column {
            if (reference.related.isEmpty()) BodyText("", Res.string.section_empty_related)
            reference.related.forEach { related ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.fillMaxWidth().clickable { onEvent(LibraryEvent.OnReferenceClick(related.id)) },
                ) {
                    Column(Modifier.weight(1f).padding(vertical = 2.dp)) {
                        Text(related.title, style = MaterialTheme.typography.bodySmall, color = colors.fillPrimary, maxLines = 2)
                        Text(related.creatorYear, style = MaterialTheme.typography.labelSmall, color = colors.fillSecondary)
                    }
                    RemoveButton { onEvent(LibraryEvent.OnUnrelate(related.id)) }
                }
            }
        }
    }
}

@Composable
private fun EditFields(draft: EditDraft, onChange: (EditDraft) -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(Dimens.SpaceMin)) {
        EditRow(stringResource(Res.string.field_title), draft.title) { onChange(draft.copy(title = it)) }
        EditRow(stringResource(Res.string.field_author), draft.authors) { onChange(draft.copy(authors = it)) }
        EditRow(stringResource(Res.string.field_date), draft.date) { onChange(draft.copy(date = it)) }
        EditRow(stringResource(Res.string.field_publication), draft.containerTitle) { onChange(draft.copy(containerTitle = it)) }
        EditRow(stringResource(Res.string.field_volume), draft.volume) { onChange(draft.copy(volume = it)) }
        EditRow(stringResource(Res.string.field_issue), draft.issue) { onChange(draft.copy(issue = it)) }
        EditRow(stringResource(Res.string.field_pages), draft.pages) { onChange(draft.copy(pages = it)) }
        EditRow(stringResource(Res.string.field_doi), draft.doi) { onChange(draft.copy(doi = it)) }
        EditRow(stringResource(Res.string.section_abstract), draft.abstract) { onChange(draft.copy(abstract = it)) }
    }
}

@Composable
private fun EditRow(label: String, value: String, onChange: (String) -> Unit) {
    Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
        Text(
            label, style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary,
            textAlign = TextAlign.End, modifier = Modifier.width(LabelWidth).padding(end = Dimens.SpaceXs),
        )
        CompactTextField(value = value, onValueChange = onChange, placeholder = label, modifier = Modifier.weight(1f))
    }
}

/** A text input with an Add button (tags, notes); Enter is not bound, the button adds. */
@Composable
private fun InputRow(value: String, placeholder: String, onChange: (String) -> Unit, onAdd: () -> Unit) {
    Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
        CompactTextField(value = value, onValueChange = onChange, placeholder = placeholder, modifier = Modifier.weight(1f))
        IconButton(onClick = onAdd, enabled = value.isNotBlank(), modifier = Modifier.size(30.dp)) {
            Icon(painterResource(Res.drawable.ic_add), placeholder, modifier = Modifier.size(16.dp))
        }
    }
}

/** A text action as tall as the header pills (a [TextButton] is 48dp at least). */
@Composable
private fun Action(label: StringResource, onClick: () -> Unit) {
    TextButton(
        onClick = onClick,
        contentPadding = PaddingValues(horizontal = Dimens.SpaceXs),
        modifier = Modifier.height(30.dp),
    ) {
        Text(stringResource(label), style = MaterialTheme.typography.labelMedium)
    }
}

@Composable
private fun RemoveButton(onClick: () -> Unit) {
    IconButton(onClick = onClick, modifier = Modifier.size(24.dp)) {
        Icon(painterResource(Res.drawable.ic_close), stringResource(Res.string.item_remove), tint = QuireTheme.colors.fillSecondary, modifier = Modifier.size(12.dp))
    }
}

/** One Info row: the label right-aligned in a fixed column, like Zotero. */
@Composable
private fun Field(label: String, value: String) {
    val colors = QuireTheme.colors
    Row(Modifier.fillMaxWidth().padding(vertical = 2.dp)) {
        Text(
            label, style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary,
            textAlign = TextAlign.End, modifier = Modifier.width(LabelWidth).padding(end = Dimens.SpaceXs),
        )
        Text(value, style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary, modifier = Modifier.weight(1f))
    }
}

private val LabelWidth = 96.dp

@Composable
private fun BodyText(text: String, empty: StringResource, modifier: Modifier = Modifier) {
    val colors = QuireTheme.colors
    Text(
        text.ifEmpty { stringResource(empty) },
        style = MaterialTheme.typography.bodyMedium,
        color = if (text.isEmpty()) colors.fillSecondary else colors.fillPrimary,
        modifier = modifier,
    )
}

@Composable
private fun SideNav(onSectionClick: (ItemSection) -> Unit) {
    Column(
        Modifier.width(Dimens.SideNavWidth).fillMaxHeight().background(QuireTheme.colors.sidepane).padding(top = Dimens.SpaceXs),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        ItemSection.entries.forEach { section ->
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier
                    .size(Dimens.ToolbarButton)
                    .clip(RoundedCornerShape(Dimens.RadiusLarge))
                    .clickable { onSectionClick(section) },
            ) {
                Icon(painterResource(section.icon), stringResource(section.title), tint = section.color(), modifier = Modifier.size(Dimens.Icon))
            }
            Spacer(Modifier.height(2.dp))
        }
    }
}
