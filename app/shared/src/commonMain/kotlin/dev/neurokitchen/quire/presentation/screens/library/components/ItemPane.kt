package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
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
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.TextButton
import androidx.compose.material3.Text
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
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
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
import dev.neurokitchen.quire.resources.ic_attach_file
import dev.neurokitchen.quire.resources.ic_expand_more
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_info
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
 * Zotero 7's item pane: the title as a header, then collapsible sections, with a side bar of
 * section icons to jump to one. Without a selection it shows how many items are in view.
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
) {
    val colors = QuireTheme.colors
    if (reference == null) {
        Box(modifier.fillMaxSize().background(colors.sidepane), contentAlignment = Alignment.Center) {
            Text(
                pluralStringResource(Res.plurals.library_item_count, itemCount, itemCount),
                style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary,
            )
        }
        return
    }
    val scroll = rememberScrollState()
    val scope = rememberCoroutineScope()
    val offsets = remember(reference.id) { mutableMapOf<ItemSection, Int>() }
    Row(modifier.fillMaxSize().background(colors.background)) {
        Column(Modifier.weight(1f).fillMaxHeight().verticalScroll(scroll)) {
            Text(
                reference.title,
                style = MaterialTheme.typography.titleLarge,
                color = colors.fillPrimary,
                modifier = Modifier.padding(start = Dimens.SpaceMd, end = Dimens.SpaceMd, top = Dimens.SpaceMd),
            )
            TextButton(onClick = onOpen, modifier = Modifier.padding(start = Dimens.SpaceXs)) {
                Text(stringResource(Res.string.reader_open))
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
                Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.clickable(onClick = onOpen)) {
                    Icon(painterResource(Res.drawable.ic_picture_as_pdf), null, tint = QuireTheme.colors.accentRed, modifier = Modifier.size(Dimens.Icon))
                    BodyText(stringResource(Res.string.attachment_pdf), Res.string.section_empty_attachments, Modifier.padding(start = Dimens.SpaceXs))
                }
            } else {
                BodyText("", Res.string.section_empty_attachments)
            }
        ItemSection.Notes -> BodyText("", Res.string.section_empty_notes)
        ItemSection.Projects -> Column {
            if (reference.projects.isEmpty()) BodyText("", Res.string.section_empty_projects)
            reference.projects.forEach { name ->
                Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.height(Dimens.RowHeight)) {
                    Icon(painterResource(Res.drawable.ic_folder), null, tint = QuireTheme.colors.accent, modifier = Modifier.size(Dimens.Icon))
                    BodyText(name, Res.string.section_empty_projects, Modifier.padding(start = Dimens.SpaceXs))
                }
            }
        }
        ItemSection.Tags -> Column {
            if (reference.tags.isEmpty()) BodyText("", Res.string.section_empty_tags)
            reference.tags.forEach { tag ->
                Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.height(Dimens.RowHeight)) {
                    Icon(painterResource(Res.drawable.ic_sell), null, tint = QuireTheme.colors.fillSecondary, modifier = Modifier.size(Dimens.Icon))
                    BodyText(tag, Res.string.section_empty_tags, Modifier.padding(start = Dimens.SpaceXs))
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
