package dev.neurokitchen.quire

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.ExpressiveSprings
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.navigation.Destination
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryRoute
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.library.components.FolderHierarchySheet
import dev.neurokitchen.quire.presentation.screens.reader.ReaderRoute
import dev.neurokitchen.quire.presentation.screens.search.SearchRoute
import dev.neurokitchen.quire.presentation.screens.settings.SettingsRoute
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_add
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/** The phone and tablet app; Android and iOS show it after `initKoin` (the desktop uses `DesktopApp`). */
@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
fun App() {
    QuireTheme {
        val colors = QuireTheme.colors
        var current by rememberSaveable { mutableStateOf(Destination.Home) }
        var isHierarchySheetOpen by rememberSaveable { mutableStateOf(false) }
        var activeReaderId by rememberSaveable { mutableStateOf<String?>(null) }
        var activeReaderTitle by rememberSaveable { mutableStateOf<String?>(null) }

        val libraryViewModel: LibraryViewModel = koinViewModel()

        if (activeReaderId != null) {
            ReaderRoute(
                id = activeReaderId!!,
                title = activeReaderTitle ?: "",
                onBack = {
                    activeReaderId = null
                    activeReaderTitle = null
                },
            )
        } else {
            Scaffold(
                bottomBar = {
                    AppBottomNavigationBar(
                        current = current,
                        isHierarchyOpen = isHierarchySheetOpen,
                        onSelect = { dest ->
                            current = dest
                            isHierarchySheetOpen = false
                        },
                        onLibraryClick = {
                            isHierarchySheetOpen = true
                        },
                        onAddClick = {
                            current = Destination.Home
                            libraryViewModel.onEvent(LibraryEvent.OnShowAddDialog(true))
                        },
                    )
                },
                contentWindowInsets = WindowInsets.statusBars,
                containerColor = colors.background,
            ) { innerPadding ->
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(innerPadding),
                ) {
                    when (current) {
                        Destination.Home, Destination.Library -> LibraryRoute(
                            onOpenReference = { id, title ->
                                activeReaderId = id
                                activeReaderTitle = title
                            },
                            viewModel = libraryViewModel,
                        )
                        Destination.Search -> SearchRoute(
                            onOpenReference = { id, title ->
                                activeReaderId = id
                                activeReaderTitle = title
                            },
                            viewModel = libraryViewModel,
                        )
                        Destination.Settings -> SettingsRoute()
                    }
                }
            }

            // Global Library Hierarchy Tree Bottom Sheet (opened by bottom bar "Library" button)
            if (isHierarchySheetOpen) {
                val libraryState by libraryViewModel.state.collectAsStateWithLifecycle()
                FolderHierarchySheet(
                    currentSource = libraryState.source,
                    projects = libraryState.projects,
                    onSelectSource = {
                        libraryViewModel.onEvent(LibraryEvent.OnSourceClick(it))
                        current = Destination.Home
                        isHierarchySheetOpen = false
                    },
                    onDismiss = { isHierarchySheetOpen = false },
                )
            }
        }
    }
}

/**
 * Modern mobile bottom navigation bar inspired by Spotify, YouTube & Pocket Hoot:
 * - Seamless surface container matching background with NO harsh color mismatch or dark gaps
 * - Order: [ Home ]  [ Library ]  [ + Add ]  [ Search ]  [ Settings ]
 * - Tactile spring-animated buttons with clean icon/text presentation
 */
@Composable
private fun AppBottomNavigationBar(
    current: Destination,
    isHierarchyOpen: Boolean,
    onSelect: (Destination) -> Unit,
    onLibraryClick: () -> Unit,
    onAddClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors

    Box(
        modifier = modifier
            .fillMaxWidth()
            .background(colors.background),
    ) {
        Column(modifier = Modifier.fillMaxWidth()) {
            HorizontalDivider(
                color = colors.paneDivider.copy(alpha = 0.35f),
                thickness = 0.5.dp,
            )
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Bottom + WindowInsetsSides.Horizontal))
                    .height(68.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                // Tab 1: Home
                BottomNavItem(
                    destination = Destination.Home,
                    selected = current == Destination.Home && !isHierarchyOpen,
                    onClick = { onSelect(Destination.Home) },
                    modifier = Modifier.weight(1f),
                )

                // Tab 2: Library (Tree hierarchy)
                BottomNavItem(
                    destination = Destination.Library,
                    selected = isHierarchyOpen,
                    onClick = onLibraryClick,
                    modifier = Modifier.weight(1f),
                )

                // Center Hero Action: + Add Reference
                CenterHeroButton(
                    onClick = onAddClick,
                    modifier = Modifier.weight(1f),
                )

                // Tab 3: Search
                BottomNavItem(
                    destination = Destination.Search,
                    selected = current == Destination.Search,
                    onClick = { onSelect(Destination.Search) },
                    modifier = Modifier.weight(1f),
                )

                // Tab 4: Settings
                BottomNavItem(
                    destination = Destination.Settings,
                    selected = current == Destination.Settings,
                    onClick = { onSelect(Destination.Settings) },
                    modifier = Modifier.weight(1f),
                )
            }
        }
    }
}

@Composable
private fun BottomNavItem(
    destination: Destination,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    val label = stringResource(destination.label)
    val interactionSource = remember { MutableInteractionSource() }
    val isPressed by interactionSource.collectIsPressedAsState()
    val scale by animateFloatAsState(
        targetValue = if (isPressed) 0.90f else 1f,
        animationSpec = if (isPressed) ExpressiveSprings.immediate() else ExpressiveSprings.easyOff(),
    )

    Column(
        modifier = modifier
            .scale(scale)
            .clickable(
                interactionSource = interactionSource,
                indication = null,
                onClick = onClick,
            )
            .padding(vertical = 6.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            painter = painterResource(destination.icon),
            contentDescription = label,
            tint = if (selected) colors.accent else colors.fillSecondary,
            modifier = Modifier.size(24.dp),
        )
        Spacer(Modifier.height(4.dp))
        Text(
            text = label,
            style = MaterialTheme.typography.labelSmall.copy(
                fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                fontSize = 11.sp,
            ),
            color = if (selected) colors.fillPrimary else colors.fillSecondary,
        )
    }
}

@Composable
private fun CenterHeroButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    val interactionSource = remember { MutableInteractionSource() }
    val isPressed by interactionSource.collectIsPressedAsState()
    val scale by animateFloatAsState(
        targetValue = if (isPressed) 0.88f else 1f,
        animationSpec = if (isPressed) ExpressiveSprings.immediate() else ExpressiveSprings.easyOff(),
    )

    val primaryColor = colors.accent
    val lightPrimary = Color(
        red = (primaryColor.red * 1.25f).coerceIn(0f, 1f),
        green = (primaryColor.green * 1.25f).coerceIn(0f, 1f),
        blue = (primaryColor.blue * 1.25f).coerceIn(0f, 1f),
        alpha = 1f,
    )
    val darkPrimary = Color(
        red = (primaryColor.red * 0.75f).coerceIn(0f, 1f),
        green = (primaryColor.green * 0.75f).coerceIn(0f, 1f),
        blue = (primaryColor.blue * 0.75f).coerceIn(0f, 1f),
        alpha = 1f,
    )

    val fabGradient = Brush.verticalGradient(
        listOf(lightPrimary, primaryColor, darkPrimary),
    )

    Box(
        modifier = modifier,
        contentAlignment = Alignment.Center,
    ) {
        Box(
            modifier = Modifier
                .size(52.dp)
                .scale(scale)
                .shadow(
                    elevation = if (isPressed) 2.dp else 4.dp,
                    shape = RoundedCornerShape(16.dp),
                    clip = false,
                )
                .clip(RoundedCornerShape(16.dp))
                .background(fabGradient)
                .clickable(
                    interactionSource = interactionSource,
                    indication = null,
                    onClick = onClick,
                ),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                painter = painterResource(Res.drawable.ic_add),
                contentDescription = "Add Reference",
                tint = colors.onAccent,
                modifier = Modifier.size(26.dp),
            )
        }
    }
}
