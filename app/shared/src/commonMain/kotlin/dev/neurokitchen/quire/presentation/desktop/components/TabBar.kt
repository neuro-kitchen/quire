package dev.neurokitchen.quire.presentation.desktop.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.ToolbarIconButton
import dev.neurokitchen.quire.presentation.desktop.OpenPaper
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_sync
import dev.neurokitchen.quire.resources.library_my_library
import dev.neurokitchen.quire.resources.library_sync
import dev.neurokitchen.quire.resources.reader_close_tab
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/** Zotero's tab bar: the library tab, a tab per open paper, and the sync button on the right. */
@Composable
fun TabBar(
    papers: List<OpenPaper>,
    /** `null`: the library tab. */
    selected: String?,
    onSelect: (String?) -> Unit,
    onClose: (String) -> Unit,
    onSyncClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(2.dp),
        modifier = modifier
            .fillMaxWidth()
            .height(Dimens.TabBarHeight)
            .background(QuireTheme.colors.tabbar)
            .padding(horizontal = Dimens.SpaceXs),
    ) {
        Tab(Res.drawable.ic_library_books, stringResource(Res.string.library_my_library), selected == null, onClick = { onSelect(null) })
        papers.forEach { paper ->
            Tab(Res.drawable.ic_description, paper.title, selected == paper.id, onClick = { onSelect(paper.id) }, onClose = { onClose(paper.id) })
        }
        Spacer(Modifier.weight(1f))
        ToolbarIconButton(Res.drawable.ic_sync, stringResource(Res.string.library_sync), onClick = onSyncClick)
    }
}

@Composable
private fun Tab(
    icon: DrawableResource,
    title: String,
    isSelected: Boolean,
    onClick: () -> Unit,
    onClose: (() -> Unit)? = null,
) {
    val colors = QuireTheme.colors
    // The open tab stands out from the bar: white on light grey, lighter grey on dark.
    val background = if (!isSelected) Color.Transparent else if (colors.isDark) colors.sidepane else colors.background
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxHeight()
            .padding(vertical = 4.dp)
            .widthIn(min = 120.dp, max = 240.dp)
            .clip(RoundedCornerShape(Dimens.RadiusLarge))
            .background(background)
            .clickable(onClick = onClick)
            .padding(horizontal = Dimens.SpaceXs),
    ) {
        Icon(painterResource(icon), null, tint = colors.accent, modifier = Modifier.size(Dimens.Icon))
        Text(
            title,
            style = MaterialTheme.typography.bodyMedium,
            color = colors.fillPrimary,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(start = 6.dp).weight(1f, fill = false),
        )
        if (onClose != null) {
            Icon(
                painterResource(Res.drawable.ic_close),
                stringResource(Res.string.reader_close_tab),
                tint = colors.fillSecondary,
                modifier = Modifier.padding(start = 4.dp).size(14.dp).clickable(onClick = onClose),
            )
        }
    }
}
