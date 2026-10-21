package dev.neurokitchen.quire.presentation.screens.tools

import dev.neurokitchen.quire.presentation.designsystem.components.DialogConfirm
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.presentation.components.notice.NoticeBar
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.tools.components.ChecksSection
import dev.neurokitchen.quire.presentation.screens.tools.components.DuplicatesSection
import dev.neurokitchen.quire.presentation.screens.tools.components.ExportSection
import dev.neurokitchen.quire.presentation.screens.tools.components.ImportSection
import dev.neurokitchen.quire.presentation.screens.tools.components.LibrariesSection
import dev.neurokitchen.quire.presentation.screens.tools.components.SyncSection
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_content_copy
import dev.neurokitchen.quire.resources.ic_folder_open
import dev.neurokitchen.quire.resources.ic_inventory_2
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_subject
import dev.neurokitchen.quire.resources.ic_sync
import dev.neurokitchen.quire.resources.tools_confirm
import dev.neurokitchen.quire.resources.tools_confirm_key_body
import dev.neurokitchen.quire.resources.tools_confirm_key_title
import dev.neurokitchen.quire.resources.tools_confirm_merge_body
import dev.neurokitchen.quire.resources.tools_confirm_merge_title
import dev.neurokitchen.quire.resources.tools_merge_confirm
import dev.neurokitchen.quire.resources.tools_section_checks
import dev.neurokitchen.quire.resources.tools_section_duplicates
import dev.neurokitchen.quire.resources.tools_section_export
import dev.neurokitchen.quire.resources.tools_section_import
import dev.neurokitchen.quire.resources.tools_section_libraries
import dev.neurokitchen.quire.resources.tools_section_sync
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

@Composable
fun ToolsRoute(viewModel: ToolsViewModel = koinViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    ToolsScreen(state, viewModel::onEvent)
}

/** The tools tab: a side list of sections, the section on the right. */
@Composable
fun ToolsScreen(state: ToolsState, onEvent: (ToolsEvent) -> Unit, modifier: Modifier = Modifier) {
    val colors = QuireTheme.colors
    Box(modifier.fillMaxSize()) {
        Row(Modifier.fillMaxSize()) {
            Column(
                Modifier.width(200.dp).fillMaxHeight().background(colors.sidepane).padding(Dimens.SpaceXs),
            ) {
                ToolsSection.entries.forEach { section ->
                    SectionRow(section, isSelected = section == state.section) { onEvent(ToolsEvent.OnSectionSelect(section)) }
                }
            }
            Column(
                Modifier
                    .weight(1f)
                    .fillMaxHeight()
                    .verticalScroll(rememberScrollState())
                    .padding(Dimens.SpaceMd)
                    .widthIn(max = 760.dp),
            ) {
                when (state.section) {
                    ToolsSection.Duplicates -> DuplicatesSection(state, onEvent)
                    ToolsSection.Checks -> ChecksSection(state, onEvent)
                    ToolsSection.Export -> ExportSection(state, onEvent)
                    ToolsSection.Import -> ImportSection(state, onEvent)
                    ToolsSection.Libraries -> LibrariesSection(state, onEvent)
                    ToolsSection.Sync -> SyncSection(state, onEvent)
                }
            }
        }
        state.notice?.let { notice ->
            NoticeBar(
                notice = notice,
                onDismiss = { onEvent(ToolsEvent.OnDismissNotice) },
                modifier = Modifier.align(Alignment.BottomCenter).padding(Dimens.SpaceMd),
            )
        }
    }
    state.confirm?.let { confirm -> Confirm(confirm, onEvent) }
}

@Composable
private fun SectionRow(section: ToolsSection, isSelected: Boolean, onClick: () -> Unit) {
    val colors = QuireTheme.colors
    val (icon, label) = section.look()
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxWidth()
            .height(Dimens.RowHeight)
            .clip(RoundedCornerShape(Dimens.RadiusLarge))
            .background(if (isSelected) colors.accent else Color.Transparent)
            .clickable(onClick = onClick)
            .padding(horizontal = Dimens.SpaceXs),
    ) {
        Icon(
            painterResource(icon), null,
            tint = if (isSelected) colors.onAccent else colors.accent,
            modifier = Modifier.padding(end = Dimens.SpaceXs).size(Dimens.Icon),
        )
        Text(stringResource(label), style = MaterialTheme.typography.bodyMedium, color = if (isSelected) colors.onAccent else colors.fillPrimary)
    }
}

private fun ToolsSection.look(): Pair<DrawableResource, StringResource> = when (this) {
    ToolsSection.Duplicates -> Res.drawable.ic_content_copy to Res.string.tools_section_duplicates
    ToolsSection.Checks -> Res.drawable.ic_subject to Res.string.tools_section_checks
    ToolsSection.Export -> Res.drawable.ic_inventory_2 to Res.string.tools_section_export
    ToolsSection.Import -> Res.drawable.ic_folder_open to Res.string.tools_section_import
    ToolsSection.Libraries -> Res.drawable.ic_library_books to Res.string.tools_section_libraries
    ToolsSection.Sync -> Res.drawable.ic_sync to Res.string.tools_section_sync
}

@Composable
private fun Confirm(confirm: ToolsConfirm, onEvent: (ToolsEvent) -> Unit) {
    val (title, body, action) = when (confirm) {
        is ToolsConfirm.Merge -> Triple(
            stringResource(Res.string.tools_confirm_merge_title, confirm.other.title),
            stringResource(Res.string.tools_confirm_merge_body, confirm.keep.title),
            stringResource(Res.string.tools_merge_confirm),
        )
        is ToolsConfirm.ImportKey -> Triple(
            stringResource(Res.string.tools_confirm_key_title),
            stringResource(Res.string.tools_confirm_key_body),
            stringResource(Res.string.tools_confirm),
        )
    }
    DialogShell(
        title = title,
        subtitle = body,
        width = 460.dp,
        onDismiss = { onEvent(ToolsEvent.OnConfirm(null)) },
        actions = {
            DialogCancel({ onEvent(ToolsEvent.OnConfirm(null)) })
            // A merge removes the other reference: the destructive colour.
            DialogConfirm(action, { onEvent(ToolsEvent.OnConfirmed) }, destructive = confirm is ToolsConfirm.Merge)
        },
    ) {}
}
