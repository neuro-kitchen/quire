package dev.neurokitchen.quire.presentation.designsystem

import androidx.compose.ui.unit.dp

/** Sizes on the desktop, from Zotero 7 (`scss/abstracts/_variables.scss`, comfortable density). */
object Dimens {
    val SpaceMin = 4.dp
    val SpaceXs = 8.dp
    val SpaceSm = 12.dp
    val SpaceMd = 16.dp
    val SpaceLg = 24.dp

    val ToolbarHeight = 41.dp
    val ToolbarButton = 28.dp
    val TabBarHeight = 36.dp
    /** One text line (20) plus Zotero's comfortable padding (11) ≈ 28 after rounding to its grid. */
    val RowHeight = 28.dp
    val HeaderHeight = 28.dp
    val Icon = 16.dp
    val RadiusSmall = 3.dp
    val RadiusBase = 4.dp
    val RadiusLarge = 6.dp

    val ProjectsPaneWidth = 250.dp
    val ItemPaneWidth = 360.dp
    val SideNavWidth = 37.dp
    val AttachmentColumnWidth = 32.dp
    val YearColumnWidth = 64.dp
}
