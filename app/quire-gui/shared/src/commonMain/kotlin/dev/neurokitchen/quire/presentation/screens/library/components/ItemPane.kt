package dev.neurokitchen.quire.presentation.screens.library.components

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

                // Action pills row
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
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

                    // Reading status badge
                    val (statusBg, statusFg, statusText) = when (reference.reading) {
                        ReadingStatus.Read -> Triple(colors.statusRead.copy(alpha = 0.15f), colors.statusRead, "Read")
                        ReadingStatus.Reading -> Triple(colors.statusReading.copy(alpha = 0.15f), colors.statusReading, "Reading")
                        ReadingStatus.Unread -> Triple(colors.fillQuarternary, colors.fillSecondary, "Unread")
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

                    Spacer(Modifier.weight(1f))

                    IconButton(
                        onClick = onDelete,
                        modifier = Modifier.size(30.dp),
                    ) {
                        Icon(painterResource(Res.drawable.ic_delete), "Delete Reference (Del)", tint = colors.fillSecondary, modifier = Modifier.size(16.dp))
                    }

                    if (onCollapse != null) {
                        IconButton(
                            onClick = onCollapse,
                            modifier = Modifier.size(30.dp),
                        ) {
                            Icon(painterResource(Res.drawable.ic_close), "Minimize Inspector (Ctrl+I)", tint = colors.fillSecondary, modifier = Modifier.size(16.dp))
                        }
                    }
                }
            }

            HorizontalDivider(color = colors.fillQuarternary)
            ItemSection.entries.forEach { section ->
                Column(Modifier.onGloballyPositioned { offsets[section] = it.positionInParent().y.toInt() }) {
                    Section(section, isCollapsed = section in collapsed, onToggle = { onSectionToggle(section) }) {
                        SectionBody(section, reference, onOpen)
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
                ShortcutRow("Ctrl + A / Ctrl + N", "Add Reference Modal")
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

@Composable
private fun SectionBody(section: ItemSection, reference: ReferenceDetailState, onOpen: () -> Unit) {
    when (section) {
        ItemSection.Info -> Column {
            Field(stringResource(Res.string.field_item_type), kindName(reference.kind))
            Field(stringResource(Res.string.field_title), reference.title)
            reference.authors.forEach { Field(stringResource(Res.string.field_author), it) }
            reference.fields.forEach { Field(stringResource(it.label), it.value) }
            Field(stringResource(Res.string.field_cite_key), reference.citeKey)
        }
        ItemSection.Abstract -> BodyText(reference.abstract, Res.string.section_empty_abstract)
        ItemSection.Attachments ->
            if (reference.hasPdf) {
                Card(
                    shape = RoundedCornerShape(Dimens.RadiusBase),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHigh),
                    modifier = Modifier.fillMaxWidth().clickable(onClick = onOpen),
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier.padding(Dimens.SpaceSm),
                    ) {
                        Icon(painterResource(Res.drawable.ic_picture_as_pdf), null, tint = QuireTheme.colors.accentRed, modifier = Modifier.size(24.dp))
                        Spacer(Modifier.width(Dimens.SpaceSm))
                        Column(Modifier.weight(1f)) {
                            Text(reference.title + ".pdf", style = MaterialTheme.typography.bodySmall, fontWeight = FontWeight.SemiBold, maxLines = 1)
                            Text("Full text PDF document", style = MaterialTheme.typography.labelSmall, color = QuireTheme.colors.fillSecondary)
                        }
                        Text("View", style = MaterialTheme.typography.labelMedium, color = QuireTheme.colors.accent, fontWeight = FontWeight.SemiBold)
                    }
                }
            } else {
                BodyText("", Res.string.section_empty_attachments)
            }
        ItemSection.Notes -> {
            Card(
                shape = RoundedCornerShape(Dimens.RadiusBase),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHigh),
                modifier = Modifier.fillMaxWidth(),
            ) {
                Column(Modifier.padding(Dimens.SpaceSm)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(painterResource(Res.drawable.ic_sticky_note_2), null, tint = QuireTheme.colors.sectionNotes, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(Dimens.SpaceMin))
                        Text("Reading Notes", style = MaterialTheme.typography.labelSmall, color = QuireTheme.colors.fillSecondary)
                    }
                    Spacer(Modifier.height(Dimens.SpaceMin))
                    Text(
                        "Detailed reading notes, mathematical derivation checks, and follow-up ideas are saved in the Reader notes panel.",
                        style = MaterialTheme.typography.bodySmall,
                        color = QuireTheme.colors.fillPrimary,
                    )
                }
            }
        }
        ItemSection.Projects -> Column {
            if (reference.projects.isEmpty()) BodyText("", Res.string.section_empty_projects)
            reference.projects.forEach { name ->
                Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.height(Dimens.RowHeight)) {
                    Icon(painterResource(Res.drawable.ic_folder), null, tint = QuireTheme.colors.accent, modifier = Modifier.size(Dimens.Icon))
                    BodyText(name, Res.string.section_empty_projects, Modifier.padding(start = Dimens.SpaceXs))
                }
            }
        }
        ItemSection.Tags -> {
            if (reference.tags.isEmpty()) {
                BodyText("", Res.string.section_empty_tags)
            } else {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    reference.tags.forEach { tag ->
                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusFull))
                                .background(QuireTheme.colors.accent.copy(alpha = 0.12f))
                                .padding(horizontal = 8.dp, vertical = 3.dp),
                        ) {
                            Text(
                                text = "#$tag",
                                style = MaterialTheme.typography.labelSmall,
                                color = QuireTheme.colors.accent,
                                fontWeight = FontWeight.Medium,
                            )
                        }
                    }
                }
            }
        }
        ItemSection.Related -> BodyText("", Res.string.section_empty_related)
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
