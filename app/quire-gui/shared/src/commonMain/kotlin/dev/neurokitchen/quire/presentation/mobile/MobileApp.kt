package dev.neurokitchen.quire.presentation.mobile

import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.ui.input.pointer.pointerInput
import kotlin.math.abs
import dev.neurokitchen.quire.presentation.screens.library.AddTab
import dev.neurokitchen.quire.presentation.mobile.components.AddFabMenu
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.contentDescription
import dev.neurokitchen.quire.presentation.mobile.components.RailWidth
import dev.neurokitchen.quire.presentation.mobile.components.MobileNavigationRail
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import dev.neurokitchen.quire.presentation.mobile.components.MinimizedBarHeight
import dev.neurokitchen.quire.presentation.mobile.components.ShortBarHeight
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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.scene.Scene
import androidx.navigation3.ui.NavDisplay
import dev.chrisbanes.haze.rememberHazeState
import dev.neurokitchen.quire.domain.model.Creator
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.PreferencesRepository
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mobile.components.AddDoiOrLinkDialog
import dev.neurokitchen.quire.presentation.mobile.components.AddFabChoice
import dev.neurokitchen.quire.presentation.mobile.components.AddFabMenu
import dev.neurokitchen.quire.presentation.mobile.components.AddFilePromptSheet
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
import dev.neurokitchen.quire.presentation.platform.PickKind
import dev.neurokitchen.quire.presentation.platform.SystemBarsAppearance
import dev.neurokitchen.quire.presentation.platform.rememberFilePicker
import dev.neurokitchen.quire.presentation.screens.reader.ReaderRoute
import dev.neurokitchen.quire.presentation.mobile.screens.SettingsPageScreen
import dev.neurokitchen.quire.presentation.mobile.screens.SettingsTab
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.add_title
import dev.neurokitchen.quire.resources.ic_add
import kotlinx.coroutines.launch
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
    val library = koinInject<LibraryRepository>()
    val scope = rememberCoroutineScope()
    val prefs by preferences.preferences.collectAsStateWithLifecycle()

    var showDoiDialog by rememberSaveable { mutableStateOf(false) }
    var showFilePrompt by rememberSaveable { mutableStateOf(false) }

    val filePicker = rememberFilePicker { kind, paths ->
        scope.launch {
            when (kind) {
                PickKind.Pdfs -> {
                    paths.forEachIndexed { index, path ->
                        val name = path.substringAfterLast('/').removeSuffix(".pdf")
                        val title = name.replace('-', ' ').replace('_', ' ')
                            .split(' ')
                            .filter { it.isNotBlank() }
                            .joinToString(" ") { word -> word.replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() } }
                        val id = "imported_pdf_${kotlin.random.Random.nextInt(1000, 9999)}_$index"
                        val ref = Reference(
                            id = id,
                            citeKey = name.lowercase().filter { it.isLetterOrDigit() }.take(10).ifBlank { "doc_$index" },
                            kind = "article-journal",
                            title = title.ifBlank { "Imported Document" },
                            creators = listOf(Creator(sortName = "Researcher", displayName = "Researcher, A.")),
                            year = 2025,
                            hasPdf = true,
                            dateAdded = "2026-10-22",
                        )
                        library.addReference(ref)
                    }
                }
                PickKind.Bib -> {
                    paths.forEachIndexed { index, path ->
                        val name = path.substringAfterLast('/').removeSuffix(".bib")
                        val id = "imported_bib_${kotlin.random.Random.nextInt(1000, 9999)}_$index"
                        val ref = Reference(
                            id = id,
                            citeKey = name.lowercase().filter { it.isLetterOrDigit() }.take(10).ifBlank { "bib_$index" },
                            kind = "article-journal",
                            title = "BibTeX: ${name.replace('-', ' ').replace('_', ' ')}",
                            creators = listOf(Creator(sortName = "Author", displayName = "Author, B.")),
                            year = 2024,
                            hasPdf = false,
                            dateAdded = "2026-10-22",
                        )
                        library.addReference(ref)
                    }
                }
                PickKind.Folder -> Unit
            }
        }
    }

    if (showDoiDialog) {
        AddDoiOrLinkDialog(
            onDismiss = { showDoiDialog = false },
            onConfirm = { input ->
                showDoiDialog = false
                scope.launch {
                    library.addByIdentifier(input)
                }
            },
        )
    }

    if (showFilePrompt) {
        AddFilePromptSheet(
            onDismiss = { showFilePrompt = false },
            onPickPdf = { filePicker.pick(PickKind.Pdfs) },
            onPickBib = { filePicker.pick(PickKind.Bib) },
        )
    }

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
            val isLandscape = maxWidth > maxHeight
            // When rotated in landscape, menu is at the bottom (YouTube style) and minimized.
            // Only large tablets in portrait orientation use the side rail.
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

            CompositionLocalProvider(LocalMobileChrome provides chrome) {
                Box(
                    Modifier
                        .fillMaxSize()
                        .padding(start = if (showBar && wide) RailWidth else 0.dp)
                        .nestedScroll(nestedScrollConnection)
                        // Swipe left/right on a tab's root to change tab (Home → Library → …).
                        // Children that scroll sideways consume the drag first.
                        .then(if (showBar) Modifier.tabSwipe(navigation) else Modifier)
                ) {
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
                                    AddFabChoice.DoiOrLink -> showDoiDialog = true
                                    AddFabChoice.File -> showFilePrompt = true
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

/** Changes tab after a horizontal drag of at least [TabSwipeDistance]. */
private fun Modifier.tabSwipe(navigation: MobileNavigation): Modifier = pointerInput(navigation) {
    val threshold = TabSwipeDistance.toPx()
    var dragged = 0f
    detectHorizontalDragGestures(
        onDragStart = { dragged = 0f },
        onDragEnd = { if (abs(dragged) >= threshold) navigation.swipe(forward = dragged < 0) },
        onHorizontalDrag = { change, dx ->
            change.consume()
            dragged += dx
        },
    )
}

private val TabSwipeDistance = 72.dp

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
        LibraryTab(
            onOpenDetails = openDetails,
            onOpenReader = openReader,
            onOpenShelf = { shelf -> navigation.open { Route.Shelf(it, shelf) } },
        )
    }
    entry<Route.Search>(metadata = TabRootTransitions) {
        SearchTab(onOpenDetails = openDetails, onOpenReader = openReader)
    }
    entry<Route.Settings>(metadata = TabRootTransitions) {
        SettingsTab(onOpen = { page -> navigation.open { Route.SettingsPage(it, page) } })
    }
    entry<Route.SettingsPage> { key ->
        SettingsPageScreen(
            key.page,
            onBack = navigation::back,
            onOpenDetails = openDetails,
            onOpenReader = openReader,
        )
    }
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
