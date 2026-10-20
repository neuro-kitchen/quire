package dev.neurokitchen.quire.presentation.desktop.components

import dev.neurokitchen.quire.resources.ic_inventory_2
import dev.neurokitchen.quire.resources.tools_menu
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.ToolbarIconButton
import dev.neurokitchen.quire.presentation.desktop.OpenPaper
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_menu
import dev.neurokitchen.quire.resources.ic_settings
import dev.neurokitchen.quire.resources.ic_sync
import dev.neurokitchen.quire.resources.library_my_library
import dev.neurokitchen.quire.resources.library_sync
import dev.neurokitchen.quire.resources.menu_settings
import dev.neurokitchen.quire.resources.reader_close_tab
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/** Quire top tab bar: hamburger app menu, brand badge, library tab, tabs for open papers, sync. */
@Composable
fun TabBar(
    papers: List<OpenPaper>,
    /** `null`: the library tab. */
    selected: String?,
    currentTheme: AppTheme = AppTheme.Obsidian,
    onSelect: (String?) -> Unit,
    onClose: (String) -> Unit,
    onSyncClick: () -> Unit,
    onSettingsClick: () -> Unit = {},
    onToolsClick: () -> Unit = {},
    onSetTheme: (AppTheme) -> Unit = {},
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    var menuOpen by remember { mutableStateOf(false) }

    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        modifier = modifier
            .fillMaxWidth()
            .height(Dimens.TabBarHeight)
            .background(colors.tabbar)
            .padding(horizontal = Dimens.SpaceXs),
    ) {
        // Hamburger Menu (replaces isolated gear on the right)
        Box {
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier
                    .size(28.dp)
                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                    .background(if (menuOpen) colors.fillQuarternary else Color.Transparent)
                    .clickable { menuOpen = true },
            ) {
                Icon(
                    painterResource(Res.drawable.ic_menu),
                    contentDescription = "Application Menu",
                    tint = colors.fillPrimary,
                    modifier = Modifier.size(16.dp),
                )
            }

            DropdownMenu(
                expanded = menuOpen,
                onDismissRequest = { menuOpen = false },
                modifier = Modifier
                    .width(260.dp)
                    .background(colors.sidepane)
                    .border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusBase)),
            ) {
                // Settings & Preferences
                DropdownMenuItem(
                    text = {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(painterResource(Res.drawable.ic_settings), contentDescription = null, tint = colors.fillSecondary, modifier = Modifier.size(16.dp))
                            Spacer(Modifier.width(8.dp))
                            Text("Preferences...", style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)
                            Spacer(Modifier.weight(1f))
                            Text("Ctrl+,", style = MaterialTheme.typography.labelSmall, color = colors.fillTertiary)
                        }
                    },
                    onClick = {
                        menuOpen = false
                        onSettingsClick()
                    },
                )

                // Tools: duplicates, checks, export, import, libraries, sync
                DropdownMenuItem(
                    text = {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(painterResource(Res.drawable.ic_inventory_2), contentDescription = null, tint = colors.fillSecondary, modifier = Modifier.size(16.dp))
                            Spacer(Modifier.width(8.dp))
                            Text(stringResource(Res.string.tools_menu), style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)
                        }
                    },
                    onClick = {
                        menuOpen = false
                        onToolsClick()
                    },
                )

                HorizontalDivider(color = colors.fillQuarternary, modifier = Modifier.padding(vertical = 4.dp))

                // Theme Sub-section
                Text(
                    text = "APPLICATION THEME",
                    style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp, fontWeight = FontWeight.Bold),
                    color = colors.fillTertiary,
                    modifier = Modifier.padding(horizontal = 12.dp, vertical = 4.dp),
                )

                AppTheme.entries.forEach { theme ->
                    val isCurrent = currentTheme == theme
                    DropdownMenuItem(
                        text = {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Box(
                                    modifier = Modifier
                                        .size(12.dp)
                                        .clip(CircleShape)
                                        .background(dev.neurokitchen.quire.presentation.designsystem.quireColors(theme).accent),
                                )
                                Spacer(Modifier.width(8.dp))
                                Text(
                                    text = theme.title,
                                    style = MaterialTheme.typography.bodyMedium.copy(
                                        fontWeight = if (isCurrent) FontWeight.Bold else FontWeight.Normal,
                                    ),
                                    color = if (isCurrent) colors.accent else colors.fillPrimary,
                                )
                                Spacer(Modifier.weight(1f))
                                if (isCurrent) {
                                    Text("✓", color = colors.accent, fontWeight = FontWeight.Bold)
                                }
                            }
                        },
                        onClick = {
                            onSetTheme(theme)
                            menuOpen = false
                        },
                    )
                }

                HorizontalDivider(color = colors.fillQuarternary, modifier = Modifier.padding(vertical = 4.dp))

                // Shortcuts Reference
                DropdownMenuItem(
                    text = {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Text("Keyboard Shortcuts...", style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)
                            Spacer(Modifier.weight(1f))
                            Text("Ctrl+/", style = MaterialTheme.typography.labelSmall, color = colors.fillTertiary)
                        }
                    },
                    onClick = {
                        menuOpen = false
                        onSettingsClick()
                    },
                )

                // Sync Library
                DropdownMenuItem(
                    text = {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(painterResource(Res.drawable.ic_sync), contentDescription = null, tint = colors.fillSecondary, modifier = Modifier.size(16.dp))
                            Spacer(Modifier.width(8.dp))
                            Text("Sync Library", style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)
                            Spacer(Modifier.weight(1f))
                            Text("Ctrl+R", style = MaterialTheme.typography.labelSmall, color = colors.fillTertiary)
                        }
                    },
                    onClick = {
                        menuOpen = false
                        onSyncClick()
                    },
                )
            }
        }

        // Quire Brand Mark
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier
                .padding(end = Dimens.SpaceSm, start = 2.dp),
        ) {
            Box(
                modifier = Modifier
                    .size(22.dp)
                    .clip(RoundedCornerShape(6.dp))
                    .background(colors.accent),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "Q",
                    style = MaterialTheme.typography.titleSmall.copy(
                        fontWeight = FontWeight.Black,
                        fontSize = 13.sp,
                        color = Color.White,
                    ),
                )
            }
            Spacer(Modifier.width(6.dp))
            Text(
                text = "QUIRE",
                style = MaterialTheme.typography.labelMedium.copy(
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 2.sp,
                    color = colors.fillPrimary,
                ),
            )
        }

        Tab(Res.drawable.ic_library_books, stringResource(Res.string.library_my_library), selected == null, onClick = { onSelect(null) })
        papers.forEach { paper ->
            Tab(Res.drawable.ic_description, paper.title, selected == paper.id, onClick = { onSelect(paper.id) }, onClose = { onClose(paper.id) })
        }
        Spacer(Modifier.weight(1f))
        ToolbarIconButton(Res.drawable.ic_sync, stringResource(Res.string.library_sync) + " (Ctrl+R)", onClick = onSyncClick)
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
    val background = when {
        isSelected -> if (colors.isDark) colors.sidepane else colors.background
        else -> Color.Transparent
    }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxHeight()
            .padding(vertical = 4.dp)
            .widthIn(min = 120.dp, max = 240.dp)
            .clip(RoundedCornerShape(Dimens.RadiusBase))
            .background(background)
            .then(
                if (isSelected) Modifier.border(1.dp, colors.fillQuarternary, RoundedCornerShape(Dimens.RadiusBase))
                else Modifier
            )
            .clickable(onClick = onClick)
            .padding(horizontal = Dimens.SpaceXs),
    ) {
        if (isSelected) {
            Box(
                modifier = Modifier
                    .size(4.dp)
                    .clip(CircleShape)
                    .background(colors.accent)
            )
            Spacer(Modifier.width(4.dp))
        }
        Icon(
            painterResource(icon),
            contentDescription = null,
            tint = if (isSelected) colors.accent else colors.fillSecondary,
            modifier = Modifier.size(Dimens.Icon),
        )
        Text(
            title,
            style = MaterialTheme.typography.bodyMedium.copy(
                fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal,
            ),
            color = if (isSelected) colors.fillPrimary else colors.fillSecondary,
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
