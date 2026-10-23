package dev.neurokitchen.quire.presentation.mobile

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.ContentTransform
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.scene.Scene
import androidx.navigation3.ui.NavDisplay
import dev.chrisbanes.haze.rememberHazeState
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mobile.components.AddDoiOrLinkDialog
import dev.neurokitchen.quire.presentation.mobile.components.AddFabChoice
import dev.neurokitchen.quire.presentation.mobile.components.AddFabMenu
import dev.neurokitchen.quire.presentation.mobile.components.AddFilePromptSheet
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MinimizedBarHeight
import dev.neurokitchen.quire.presentation.mobile.components.MobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileNavigationBar
import dev.neurokitchen.quire.presentation.mobile.components.MobileNavigationRail
import dev.neurokitchen.quire.presentation.mobile.components.RailWidth
import dev.neurokitchen.quire.presentation.mobile.components.ShortBarHeight
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
import dev.neurokitchen.quire.presentation.mobile.screens.SettingsPageScreen
import dev.neurokitchen.quire.presentation.mobile.screens.SettingsTab
import dev.neurokitchen.quire.presentation.mobile.screens.ShelfScreen
import dev.neurokitchen.quire.presentation.mobile.share.ShareInbox
import dev.neurokitchen.quire.presentation.mobile.share.SharedItem
import dev.neurokitchen.quire.presentation.mobile.share.sharedIdentifiers
import dev.neurokitchen.quire.presentation.platform.PickKind
import dev.neurokitchen.quire.presentation.platform.SystemBarsAppearance
import dev.neurokitchen.quire.presentation.platform.rememberFilePicker
import dev.neurokitchen.quire.presentation.screens.library.AddTab
import dev.neurokitchen.quire.presentation.screens.reader.ReaderRoute
import org.koin.compose.viewmodel.koinViewModel

/**
 * The phone and tablet app: four tabs hosted in an interactive finger-tracking [HorizontalPager]
 * (like WhatsApp and Instagram), with each tab keeping its own back stack (Navigation 3) for
 * pushed sub-screens. Back pops the current tab, then returns to Home.
 */
@Composable
fun MobileApp(
    viewModel: MobileAppViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()

    val filePicker = rememberFilePicker { kind, paths ->
        viewModel.onEvent(MobileAppEvent.OnImportFiles(kind, paths))
    }

    if (state.showDoiDialog) {
        AddDoiOrLinkDialog(
            onDismiss = { viewModel.onEvent(MobileAppEvent.OnDismissDoiDialog) },
            onConfirm = { input -> viewModel.onEvent(MobileAppEvent.OnAddByIdentifier(input)) },
        )
    }

    if (state.showFilePrompt) {
        AddFilePromptSheet(
            onDismiss = { viewModel.onEvent(MobileAppEvent.OnDismissFilePrompt) },
            onPickPdf = { filePicker.pick(PickKind.Pdfs) },
            onPickBib = { filePicker.pick(PickKind.Bib) },
        )
    }

    QuireTheme(appTheme = state.appTheme) {
        SystemBarsAppearance(darkTheme = QuireTheme.colors.isDark)
        val navigation = rememberMobileNavigation()
        val haze = rememberHazeState()

        // Shared with Quire (Android's share sheet, "Open with"): Add, filled in.
        val shared by ShareInbox.item.collectAsStateWithLifecycle()
        LaunchedEffect(shared) {
            when (val item = ShareInbox.take()) {
                is SharedItem.Text -> navigation.open {
                    Route.Add(it, identifiers = sharedIdentifiers(item.text).joinToString("\n"))
                }
                is SharedItem.Files -> navigation.open { Route.Add(it, pdfs = item.paths) }
                null -> {}
            }
        }

        val showBar = navigation.isAtTabRoot
        val pagerState = rememberPagerState(initialPage = navigation.tab.ordinal) { Tab.entries.size }
        var programmaticTargetTab by remember { mutableStateOf<Tab?>(null) }

        // Sync programmatic tab changes (bottom bar tap, rail tap, or Back to Home) -> Pager
        LaunchedEffect(navigation.tab) {
            if (pagerState.currentPage != navigation.tab.ordinal) {
                programmaticTargetTab = navigation.tab
                try {
                    pagerState.animateScrollToPage(navigation.tab.ordinal)
                } finally {
                    programmaticTargetTab = null
                }
            }
        }

        // Sync interactive user finger-swipe on Pager -> MobileNavigation.tab
        LaunchedEffect(pagerState) {
            snapshotFlow { pagerState.currentPage }.collect { page ->
                if (programmaticTargetTab == null) {
                    val pageTab = Tab.entries.getOrNull(page)
                    if (pageTab != null && pageTab != navigation.tab) {
                        navigation.select(pageTab)
                    }
                }
            }
        }

        BoxWithConstraints(Modifier.fillMaxSize().background(QuireTheme.colors.background)) {
            val isLandscape = maxWidth > maxHeight
            val wide = !isLandscape && maxWidth >= WideWindow
            val chrome = MobileChrome(haze, bottomBar = if (showBar && !wide) bottomBarSpace(isLandscape) else 0.dp)

            val density = LocalDensity.current
            val navBarBottom = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
            val barHeight = if (isLandscape) MinimizedBarHeight else ShortBarHeight
            val totalBarPx = with(density) { (barHeight + navBarBottom + 16.dp).toPx() }
            var bottomBarOffsetPx by rememberSaveable { mutableFloatStateOf(0f) }

            val nestedScrollConnection = remember(totalBarPx, isLandscape) {
                object : NestedScrollConnection {
                    override fun onPreScroll(available: Offset, source: NestedScrollSource): Offset {
                        if (isLandscape) {
                            val delta = available.y
                            val newOffset = bottomBarOffsetPx + delta
                            bottomBarOffsetPx = newOffset.coerceIn(-totalBarPx, 0f)
                        } else {
                            bottomBarOffsetPx = 0f
                        }
                        return Offset.Zero
                    }
                }
            }

            val openDetails = remember(navigation) { { id: String -> navigation.open { tab -> Route.Details(tab, id) } } }
            val openReader = remember(navigation) { { id: String, title: String -> navigation.open { tab -> Route.Reader(tab, id, title) } } }
            val openShelf = remember(navigation) { { shelf: dev.neurokitchen.quire.presentation.mobile.navigation.ShelfKey -> navigation.open { Route.Shelf(it, shelf) } } }

            CompositionLocalProvider(LocalMobileChrome provides chrome) {
                Box(
                    Modifier
                        .fillMaxSize()
                        .padding(start = if (showBar && wide) RailWidth else 0.dp)
                        .nestedScroll(nestedScrollConnection),
                ) {
                    // Persistent finger-tracking HorizontalPager for the 4 root tabs (WhatsApp / Instagram style)
                    HorizontalPager(
                        state = pagerState,
                        userScrollEnabled = showBar,
                        beyondViewportPageCount = 3,
                        modifier = Modifier.fillMaxSize(),
                    ) { page ->
                        when (Tab.entries[page]) {
                            Tab.Home -> HomeTab(
                                onOpenDetails = openDetails,
                                onOpenReader = openReader,
                                onOpenShelf = openShelf,
                                onOpenNotifications = {
                                    navigation.open { Route.SettingsPage(it, dev.neurokitchen.quire.presentation.mobile.navigation.MobileSettingsPage.Notifications) }
                                },
                            )
                            Tab.Library -> LibraryTab(
                                onOpenDetails = openDetails,
                                onOpenReader = openReader,
                                onOpenShelf = openShelf,
                            )
                            Tab.Search -> SearchTab(
                                onOpenDetails = openDetails,
                                onOpenReader = openReader,
                            )
                            Tab.Settings -> SettingsTab(
                                onOpen = { settingsPage ->
                                    navigation.open { Route.SettingsPage(it, settingsPage) }
                                },
                            )
                        }
                    }

                    // Overlay NavDisplay for pushed sub-screens (Details, Reader, Shelf, SettingsPage, Add)
                    // and Back handling across tab stacks.
                    NavDisplay(
                        entries = navigation.entries(entryProvider(navigation)),
                        onBack = navigation::back,
                        modifier = Modifier.fillMaxSize(),
                        transitionSpec = { push() },
                        popTransitionSpec = { pop() },
                        predictivePopTransitionSpec = { pop() },
                    )

                    if (showBar && (navigation.tab == Tab.Home || navigation.tab == Tab.Library)) {
                        AddFabMenu(
                            onSelectChoice = { choice ->
                                when (choice) {
                                    AddFabChoice.DoiOrLink -> viewModel.onEvent(MobileAppEvent.OnShowDoiDialog)
                                    AddFabChoice.File -> viewModel.onEvent(MobileAppEvent.OnShowFilePrompt)
                                }
                            },
                            bottom = chrome.bottomBar,
                            modifier = Modifier.graphicsLayer {
                                translationY = -bottomBarOffsetPx
                            },
                        )
                    }
                    if (showBar && !wide) {
                        MobileNavigationBar(
                            current = navigation.tab,
                            onSelect = navigation::select,
                            haze = haze,
                            isLandscape = isLandscape,
                            modifier = Modifier
                                .align(Alignment.BottomCenter)
                                .graphicsLayer {
                                    translationY = -bottomBarOffsetPx
                                },
                        )
                    }
                }
                if (showBar && wide) {
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

    // Tab roots are rendered in the underlying HorizontalPager so they track horizontal finger drag
    // continuously; their NavDisplay entries act as transparent anchors for back-stack management.
    entry<Route.Home> {}
    entry<Route.Library> {}
    entry<Route.Search> {}
    entry<Route.Settings> {}

    entry<Route.SettingsPage> { key ->
        PushedScreenContainer {
            SettingsPageScreen(
                key.page,
                onBack = navigation::back,
                onOpenDetails = openDetails,
                onOpenReader = openReader,
            )
        }
    }
    entry<Route.Shelf> { key ->
        PushedScreenContainer {
            ShelfScreen(key.shelf, onBack = navigation::back, onOpenDetails = openDetails, onOpenReader = openReader)
        }
    }
    entry<Route.Details> { key ->
        PushedScreenContainer {
            DetailsScreen(key.id, onBack = navigation::back, onOpenReader = openReader)
        }
    }
    entry<Route.Reader> { key ->
        PushedScreenContainer {
            ReaderRoute(id = key.id, title = key.title, onBack = navigation::back)
        }
    }
    entry<Route.Add> { key ->
        PushedScreenContainer {
            AddScreen(
                projectPath = key.projectPath,
                onDone = navigation::back,
                identifiers = key.identifiers,
                pdfs = key.pdfs,
                way = key.way?.let { name -> AddTab.entries.firstOrNull { it.name == name } },
            )
        }
    }
}

/** Blocks touch bleed-through to the underlying [HorizontalPager] when a sub-screen is pushed. */
@Composable
private fun PushedScreenContainer(content: @Composable () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .pointerInput(Unit) {},
    ) {
        content()
    }
}

/** A pushed screen slides in from the end over the underlying tab pager. */
private fun AnimatedContentTransitionScope<Scene<NavKey>>.push(): ContentTransform =
    (slideInHorizontally(tween(300)) { it } + fadeIn(tween(300))) togetherWith
        (slideOutHorizontally(tween(300)) { -it / 4 } + fadeOut(tween(300)))

/** Back: the pushed screen slides out to the end (also while predictive swipe-back is in progress). */
private fun AnimatedContentTransitionScope<Scene<NavKey>>.pop(): ContentTransform =
    (slideInHorizontally(tween(300)) { -it / 4 } + fadeIn(tween(300))) togetherWith
        (slideOutHorizontally(tween(300)) { it } + fadeOut(tween(300)))
