package dev.neurokitchen.quire.presentation.mobile

import dev.neurokitchen.quire.presentation.screens.library.AddTab
import dev.neurokitchen.quire.presentation.mobile.components.AddFabMenu
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.contentDescription
import dev.neurokitchen.quire.presentation.mobile.components.RailWidth
import dev.neurokitchen.quire.presentation.mobile.components.MobileNavigationRail
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.BoxWithConstraints
import dev.neurokitchen.quire.presentation.mobile.share.sharedIdentifiers
import dev.neurokitchen.quire.presentation.mobile.share.SharedItem
import dev.neurokitchen.quire.presentation.mobile.share.ShareInbox
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.ContentTransform
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.scene.Scene
import androidx.navigation3.ui.NavDisplay
import dev.chrisbanes.haze.rememberHazeState
import dev.neurokitchen.quire.domain.repository.PreferencesRepository
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileNavigationBar
import dev.neurokitchen.quire.presentation.mobile.components.bottomBarSpace
import dev.neurokitchen.quire.presentation.mobile.navigation.MobileNavigation
import dev.neurokitchen.quire.presentation.mobile.navigation.Route
import dev.neurokitchen.quire.presentation.mobile.navigation.Tab
import dev.neurokitchen.quire.presentation.mobile.navigation.entries
import dev.neurokitchen.quire.presentation.mobile.navigation.rememberMobileNavigation
import dev.neurokitchen.quire.presentation.mobile.screens.AddScreen
import dev.neurokitchen.quire.presentation.mobile.screens.DetailsScreen
import dev.neurokitchen.quire.presentation.mobile.screens.HomeTab
import dev.neurokitchen.quire.presentation.mobile.screens.LibraryTab
import dev.neurokitchen.quire.presentation.mobile.screens.SearchTab
import dev.neurokitchen.quire.presentation.mobile.screens.ShelfScreen
import dev.neurokitchen.quire.presentation.platform.SystemBarsAppearance
import dev.neurokitchen.quire.presentation.screens.reader.ReaderRoute
import dev.neurokitchen.quire.presentation.mobile.screens.SettingsPageScreen
import dev.neurokitchen.quire.presentation.mobile.screens.SettingsTab
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.add_title
import dev.neurokitchen.quire.resources.ic_add
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.koinInject

/**
 * The phone and tablet app: four tabs, each with its own back stack (Navigation 3). Back, the
 * system's swipe from either edge included (predictive back: the previous screen shows as you
 * swipe), pops the current tab, then returns to Home. The bottom bar is frosted (Haze) over the
 * content and shown on the tabs' roots only; screens opened from them use their own top bar.
 */
@Composable
fun MobileApp() {
    val preferences = koinInject<PreferencesRepository>()
    val prefs by preferences.preferences.collectAsStateWithLifecycle()
    QuireTheme(appTheme = prefs.appTheme) {
        SystemBarsAppearance(darkTheme = QuireTheme.colors.isDark)
        val navigation = rememberMobileNavigation()
        val haze = rememberHazeState()
        // Shared with Quire (Android's share sheet, "Open with"): Add, filled in.
        val shared by ShareInbox.item.collectAsStateWithLifecycle()
        LaunchedEffect(shared) {
            when (val item = ShareInbox.take()) {
                is SharedItem.Text -> navigation.open { Route.Add(it, identifiers = sharedIdentifiers(item.text).joinToString("\n")) }
                is SharedItem.Files -> navigation.open { Route.Add(it, pdfs = item.paths) }
                null -> {}
            }
        }
        val showBar = navigation.isAtTabRoot
        BoxWithConstraints(Modifier.fillMaxSize().background(QuireTheme.colors.background)) {
            // Wide windows (tablets, unfolded phones): the tabs as a rail at the start, always shown.
            val wide = maxWidth >= WideWindow
            val chrome = MobileChrome(haze, bottomBar = if (showBar && !wide) bottomBarSpace() else 0.dp)
            CompositionLocalProvider(LocalMobileChrome provides chrome) {
                Box(Modifier.fillMaxSize().padding(start = if (wide) RailWidth else 0.dp)) {
                    NavDisplay(
                        entries = navigation.entries(entryProvider(navigation)),
                        onBack = navigation::back,
                        modifier = Modifier.fillMaxSize(),
                        transitionSpec = { push() },
                        popTransitionSpec = { pop() },
                        predictivePopTransitionSpec = { pop() },
                    )
                    if (showBar && (navigation.tab == Tab.Home || navigation.tab == Tab.Library)) {
                        AddFabMenu(onAdd = { way -> navigation.open { Route.Add(it, way = way.name) } }, bottom = chrome.bottomBar)
                    }
                    if (showBar && !wide) {
                        MobileNavigationBar(
                            current = navigation.tab,
                            onSelect = navigation::select,
                            haze = haze,
                            modifier = Modifier.align(Alignment.BottomCenter),
                        )
                    }
                }
                if (wide) {
                    MobileNavigationRail(
                        current = navigation.tab,
                        onSelect = navigation::select,
                        haze = haze,
                        modifier = Modifier.width(RailWidth).fillMaxHeight(),
                    )
                }
            }
        }
    }
}

/** Material's medium window width: from here the tabs are a rail, not a bottom bar. */
private val WideWindow = 600.dp

private fun entryProvider(navigation: MobileNavigation) = entryProvider<NavKey> {
    val openDetails = { id: String -> navigation.open { tab -> Route.Details(tab, id) } }
    val openReader = { id: String, title: String -> navigation.open { tab -> Route.Reader(tab, id, title) } }
    entry<Route.Home>(metadata = TabRootTransitions) {
        HomeTab(
            onOpenDetails = openDetails,
            onOpenReader = openReader,
            onOpenShelf = { shelf -> navigation.open { Route.Shelf(it, shelf) } },
        )
    }
    entry<Route.Library>(metadata = TabRootTransitions) {
        LibraryTab(onOpenShelf = { shelf -> navigation.open { Route.Shelf(it, shelf) } })
    }
    entry<Route.Search>(metadata = TabRootTransitions) {
        SearchTab(onOpenDetails = openDetails, onOpenReader = openReader)
    }
    entry<Route.Settings>(metadata = TabRootTransitions) {
        SettingsTab(onOpen = { page -> navigation.open { Route.SettingsPage(it, page) } })
    }
    entry<Route.SettingsPage> { key -> SettingsPageScreen(key.page, onBack = navigation::back) }
    entry<Route.Shelf> { key ->
        ShelfScreen(key.shelf, onBack = navigation::back, onOpenDetails = openDetails, onOpenReader = openReader)
    }
    entry<Route.Details> { key ->
        DetailsScreen(key.id, onBack = navigation::back, onOpenReader = openReader)
    }
    entry<Route.Reader> { key ->
        ReaderRoute(id = key.id, title = key.title, onBack = navigation::back)
    }
    entry<Route.Add> { key ->
        AddScreen(
            projectPath = key.projectPath,
            onDone = navigation::back,
            identifiers = key.identifiers,
            pdfs = key.pdfs,
            way = key.way?.let { name -> AddTab.entries.firstOrNull { it.name == name } },
        )
    }
}

/** A pushed screen slides in from the end, the one under it moves a little and dims. */
private fun AnimatedContentTransitionScope<Scene<NavKey>>.push(): ContentTransform =
    (slideInHorizontally(tween(300)) { it } + fadeIn(tween(300))) togetherWith
        (slideOutHorizontally(tween(300)) { -it / 4 } + fadeOut(tween(300)))

/** Back: the reverse (also while the swipe-back gesture is in progress). */
private fun AnimatedContentTransitionScope<Scene<NavKey>>.pop(): ContentTransform =
    (slideInHorizontally(tween(300)) { -it / 4 } + fadeIn(tween(300))) togetherWith
        (slideOutHorizontally(tween(300)) { it } + fadeOut(tween(300)))

/** Switching tabs cross-fades: a tab is a place, not a step forward. */
private val TabRootTransitions: Map<String, Any> =
    NavDisplay.transitionSpec { fadeIn(tween(200)) togetherWith fadeOut(tween(200)) } +
        NavDisplay.popTransitionSpec { fadeIn(tween(200)) togetherWith fadeOut(tween(200)) }
