package dev.neurokitchen.quire.presentation.mobile.components

import androidx.compose.material3.NavigationRail
import androidx.compose.material3.NavigationRailItem
import androidx.compose.material3.NavigationRailItemDefaults
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ShortNavigationBar
import androidx.compose.material3.ShortNavigationBarItem
import androidx.compose.material3.ShortNavigationBarItemDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dev.chrisbanes.haze.HazeInput
import dev.chrisbanes.haze.HazeState
import dev.chrisbanes.haze.blur.hazeBlur
import dev.chrisbanes.haze.blur.materials.HazeMaterials
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mobile.navigation.Tab
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.common_back
import dev.neurokitchen.quire.resources.destination_home
import dev.neurokitchen.quire.resources.destination_library
import dev.neurokitchen.quire.resources.destination_search
import dev.neurokitchen.quire.resources.destination_settings
import dev.neurokitchen.quire.resources.ic_arrow_back
import dev.neurokitchen.quire.resources.ic_home
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.ic_settings
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/**
 * What the app's chrome tells a screen: the haze its bars blur (screens mark their scrolling
 * content as its source), and how much of the bottom the navigation bar covers, so lists run
 * under it and end above it.
 */
class MobileChrome(val haze: HazeState, val bottomBar: Dp)

val LocalMobileChrome = compositionLocalOf<MobileChrome> { error("no MobileChrome: inside MobileApp only") }

/** Height of Material 3 Expressive's short navigation bar, without the system navigation bar. */
val ShortBarHeight = 64.dp

/** Height of [MobileTopBar], without the status bar. */
val TopBarHeight = 56.dp

/** Padding for a scrolling screen under a [MobileTopBar] (if [topBar]) and above the bottom bar. */
@Composable
fun mobileContentPadding(topBar: Boolean = true, extraBottom: Dp = Dimens.SpaceMd): PaddingValues {
    val status = WindowInsets.statusBars.asPaddingValues().calculateTopPadding()
    val chrome = LocalMobileChrome.current
    return PaddingValues(
        top = status + (if (topBar) TopBarHeight else 0.dp),
        bottom = chrome.bottomBar + extraBottom,
    )
}

/** The bottom bar's space: the bar and the system navigation bar under it. */
@Composable
fun bottomBarSpace(): Dp = ShortBarHeight + WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()

/**
 * The tabs, as Material 3 Expressive's short navigation bar, frosted: the content scrolls under
 * it and shows blurred through it (Haze).
 */
@Composable
fun MobileNavigationBar(
    current: Tab,
    onSelect: (Tab) -> Unit,
    haze: HazeState,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    ShortNavigationBar(
        modifier = modifier.hazeBlur(HazeInput.Sources(haze), HazeMaterials.thin(colors.background)),
        containerColor = Color.Transparent,
    ) {
        Tab.entries.forEach { tab ->
            val label = stringResource(tab.label())
            ShortNavigationBarItem(
                selected = tab == current,
                onClick = { onSelect(tab) },
                icon = { Icon(painterResource(tab.icon()), contentDescription = null) },
                label = { Text(label) },
                colors = ShortNavigationBarItemDefaults.colors(
                    selectedIconColor = colors.accent,
                    selectedTextColor = colors.fillPrimary,
                    selectedIndicatorColor = colors.accent.copy(alpha = 0.16f),
                    unselectedIconColor = colors.fillSecondary,
                    unselectedTextColor = colors.fillSecondary,
                ),
            )
        }
    }
}

/** Width of the navigation rail (tablets, unfolded phones, landscape). */
val RailWidth = 92.dp

/**
 * The tabs on wide windows (600 dp and more: tablets, unfolded phones, landscape windows),
 * as Material 3's navigation rail at the start; frosted like the bottom bar.
 */
@Composable
fun MobileNavigationRail(
    current: Tab,
    onSelect: (Tab) -> Unit,
    haze: HazeState,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    NavigationRail(
        modifier = modifier.hazeBlur(HazeInput.Sources(haze), HazeMaterials.thin(colors.background)),
        containerColor = Color.Transparent,
        contentColor = colors.fillPrimary,
    ) {
        Tab.entries.forEach { tab ->
            NavigationRailItem(
                selected = tab == current,
                onClick = { onSelect(tab) },
                icon = { Icon(painterResource(tab.icon()), contentDescription = null) },
                label = {
                    Text(
                        stringResource(tab.label()),
                        style = MaterialTheme.typography.labelSmall,
                        maxLines = 1,
                        softWrap = false,
                    )
                },
                colors = NavigationRailItemDefaults.colors(
                    selectedIconColor = colors.accent,
                    selectedTextColor = colors.fillPrimary,
                    indicatorColor = colors.accent.copy(alpha = 0.16f),
                    unselectedIconColor = colors.fillSecondary,
                    unselectedTextColor = colors.fillSecondary,
                ),
            )
        }
    }
}

/**
 * The top bar of a screen opened from a tab: back, a title, actions; frosted like the bottom bar,
 * over content that scrolls under it (padding from [mobileContentPadding]).
 */
@Composable
fun MobileTopBar(
    title: String,
    onBack: (() -> Unit)?,
    haze: HazeState,
    modifier: Modifier = Modifier,
    actions: @Composable RowScope.() -> Unit = {},
) {
    val colors = QuireTheme.colors
    Box(
        modifier
            .fillMaxWidth()
            .hazeBlur(HazeInput.Sources(haze), HazeMaterials.thin(colors.background))
            .windowInsetsPadding(WindowInsets.statusBars),
    ) {
        Row(
            Modifier.fillMaxWidth().height(TopBarHeight).padding(horizontal = Dimens.SpaceMin),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceMin),
        ) {
            if (onBack != null) {
                IconButton(onClick = onBack) {
                    Icon(painterResource(Res.drawable.ic_arrow_back), contentDescription = stringResource(Res.string.common_back), tint = colors.fillPrimary)
                }
            } else {
                Box(Modifier.size(Dimens.SpaceXs))
            }
            Text(
                title,
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.SemiBold,
                color = colors.fillPrimary,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )
            actions()
        }
    }
}

private fun Tab.label() = when (this) {
    Tab.Home -> Res.string.destination_home
    Tab.Library -> Res.string.destination_library
    Tab.Search -> Res.string.destination_search
    Tab.Settings -> Res.string.destination_settings
}

private fun Tab.icon() = when (this) {
    Tab.Home -> Res.drawable.ic_home
    Tab.Library -> Res.drawable.ic_library_books
    Tab.Search -> Res.drawable.ic_search
    Tab.Settings -> Res.drawable.ic_settings
}
