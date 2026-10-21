package dev.neurokitchen.quire.presentation.screens.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import dev.neurokitchen.quire.presentation.designsystem.quireColors
import dev.neurokitchen.quire.presentation.designsystem.readerColors
import dev.neurokitchen.quire.presentation.screens.settings.components.RawConfigEditor
import dev.neurokitchen.quire.presentation.screens.settings.components.SettingRow
import dev.neurokitchen.quire.presentation.screens.settings.components.SettingsSidebar
import dev.neurokitchen.quire.presentation.screens.settings.components.displayTitle
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_link
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import dev.neurokitchen.quire.resources.settings_file_location
import dev.neurokitchen.quire.resources.settings_load_failed
import dev.neurokitchen.quire.resources.settings_none_named
import dev.neurokitchen.quire.resources.settings_version
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

@Composable
fun SettingsRoute(viewModel: SettingsViewModel = koinViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    SettingsScreen(state, viewModel::onEvent)
}

/**
 * Quire Settings Hub:
 * - Multi-category navigation: Appearance, Reader Defaults, General & Sync, Libraries, CiteKey & Export, Shortcuts, ConfigFile
 * - Live preview cards for App Themes and Reader Defaults
 * - Keyboard shortcuts cheat-sheet
 * - Raw TOML editor with validation
 */
@Composable
fun SettingsScreen(
    state: SettingsState,
    onEvent: (SettingsEvent) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    BoxWithConstraints(modifier.fillMaxSize()) {
        val isNarrow = maxWidth < 600.dp
        if (isNarrow) {
            Column(Modifier.fillMaxSize()) {
                // Category tabs: horizontal scrollable chips
                LazyRow(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(colors.cardBackground)
                        .padding(horizontal = 12.dp, vertical = 8.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    items(SettingsSection.entries) { section ->
                        val selected = section == state.section
                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusFull))
                                .background(if (selected) colors.accent else colors.fillQuinary)
                                .border(
                                    1.dp,
                                    if (selected) colors.accent else colors.border.copy(alpha = 0.5f),
                                    RoundedCornerShape(Dimens.RadiusFull),
                                )
                                .clickable { onEvent(SettingsEvent.OnSectionClick(section)) }
                                .padding(horizontal = 12.dp, vertical = 6.dp),
                        ) {
                            Text(
                                text = section.displayTitle,
                                style = MaterialTheme.typography.labelSmall.copy(
                                    fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                                    color = if (selected) colors.onAccent else colors.fillPrimary,
                                ),
                            )
                        }
                    }
                }
                HorizontalDivider(color = colors.paneDivider)
                SettingsCategoryContent(state, onEvent, Modifier.fillMaxSize().padding(Dimens.SpaceMd))
            }
        } else {
            // Desktop side-by-side
            Row(Modifier.fillMaxSize()) {
                SettingsSidebar(state.section, onClick = { onEvent(SettingsEvent.OnSectionClick(it)) }, Modifier.width(200.dp))
                VerticalDivider(color = colors.paneDivider)
                SettingsCategoryContent(state, onEvent, Modifier.weight(1f).padding(Dimens.SpaceMd))
            }
        }
    }
}

@Composable
private fun SettingsCategoryContent(
    state: SettingsState,
    onEvent: (SettingsEvent) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    Column(modifier) {
        // Category Header
        Row(verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) {
                Text(state.section.displayTitle, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold, color = colors.fillPrimary)
                Text(
                    if (state.configPath.isNotEmpty()) stringResource(Res.string.settings_file_location, state.configPath)
                    else "Configure preferences, appearance, shortcuts, and sync",
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.fillSecondary,
                )
            }
        }
        HorizontalDivider(color = colors.fillQuarternary, modifier = Modifier.padding(vertical = Dimens.SpaceSm))

        val error = state.error
        when {
            error != null -> Column {
                Text(stringResource(Res.string.settings_load_failed), color = colors.accentRed, style = MaterialTheme.typography.titleMedium)
                Text(error, color = colors.fillSecondary, style = MaterialTheme.typography.bodyMedium)
            }
            state.section == SettingsSection.Appearance -> AppearanceSectionView(state.appTheme, onEvent)
            state.section == SettingsSection.Shortcuts -> ShortcutsSectionView()
            state.section == SettingsSection.Reader -> ReaderDefaultsSectionView(state.readerFont, state.readerTheme, state.readerFontSize, onEvent)
            state.section == SettingsSection.ConfigFile -> RawConfigEditor(
                state.raw,
                onChange = { onEvent(SettingsEvent.OnRawChange(it)) },
                onSave = { onEvent(SettingsEvent.OnRawSave) },
                onRevert = { onEvent(SettingsEvent.OnRawRevert) },
            )
            else -> LazyColumn(Modifier.weight(1f)) {
                items(state.fields, key = { it.key }) { field ->
                    SettingRow(
                        field,
                        onDraftChange = { onEvent(SettingsEvent.OnDraftChange(field.key, it)) },
                        onApply = { onEvent(SettingsEvent.OnApply(field.key)) },
                        onReset = { onEvent(SettingsEvent.OnReset(field.key)) },
                    )
                }
                if (!state.isLoading && state.fields.isEmpty()) {
                    item { Text(stringResource(Res.string.settings_none_named), color = colors.fillSecondary, style = MaterialTheme.typography.bodyMedium) }
                }
                item {
                    Text(
                        "${stringResource(Res.string.settings_version)} ${state.version}",
                        style = MaterialTheme.typography.bodySmall,
                        color = colors.fillTertiary,
                        modifier = Modifier.padding(top = Dimens.SpaceLg),
                    )
                }
            }
        }
    }
}

/** Appearance & Full App Theming preferences */
@Composable
private fun AppearanceSectionView(
    activeTheme: AppTheme,
    onEvent: (SettingsEvent) -> Unit,
) {
    val colors = QuireTheme.colors

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceMd),
    ) {
        Text("Application Theme", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = colors.fillPrimary)
        Text("Select an overarching theme for all windows, toolbars, sidebars, and dialogs.", style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary)

        // Theme Cards Grid
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            AppTheme.entries.forEach { theme ->
                val themeCol = quireColors(theme)
                val isSelected = activeTheme == theme
                Card(
                    shape = RoundedCornerShape(Dimens.RadiusBase),
                    colors = CardDefaults.cardColors(containerColor = if (isSelected) colors.accent.copy(alpha = 0.12f) else colors.sidepane),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(if (isSelected) 1.5.dp else 1.dp, if (isSelected) colors.accent else colors.border, RoundedCornerShape(Dimens.RadiusBase))
                        .clickable { onEvent(SettingsEvent.OnSetAppTheme(theme)) },
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier.padding(Dimens.SpaceSm),
                    ) {
                        // Palette Swatch
                        Row(
                            horizontalArrangement = Arrangement.spacedBy(4.dp),
                            modifier = Modifier
                                .clip(RoundedCornerShape(6.dp))
                                .background(themeCol.background)
                                .border(1.dp, themeCol.paneDivider, RoundedCornerShape(6.dp))
                                .padding(6.dp),
                        ) {
                            Box(Modifier.size(16.dp).clip(CircleShape).background(themeCol.background))
                            Box(Modifier.size(16.dp).clip(CircleShape).background(themeCol.accent))
                            Box(Modifier.size(16.dp).clip(CircleShape).background(themeCol.sidepane))
                            Box(Modifier.size(16.dp).clip(CircleShape).background(themeCol.fillPrimary))
                        }

                        Spacer(Modifier.width(Dimens.SpaceMd))

                        Column(Modifier.weight(1f)) {
                            Text(theme.title, style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.SemiBold, color = colors.fillPrimary)
                            Text(
                                if (theme.isDark) "Dark mode palette" else "Clean light editorial palette",
                                style = MaterialTheme.typography.bodySmall,
                                color = colors.fillSecondary,
                            )
                        }

                        if (isSelected) {
                            Box(
                                modifier = Modifier
                                    .clip(RoundedCornerShape(Dimens.RadiusFull))
                                    .background(colors.accent)
                                    .padding(horizontal = 8.dp, vertical = 2.dp),
                            ) {
                                Text("Active", style = MaterialTheme.typography.labelSmall, color = colors.onAccent, fontWeight = FontWeight.Bold)
                            }
                        }
                    }
                }
            }
        }
    }
}

/** Reader Defaults preferences */
@Composable
private fun ReaderDefaultsSectionView(
    selectedFont: ReaderFont,
    selectedTheme: ReaderTheme,
    defaultFontSize: Int,
    onEvent: (SettingsEvent) -> Unit,
) {
    val colors = QuireTheme.colors

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceMd),
    ) {
        Text("Default View Mode", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = colors.fillPrimary)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Card(
                shape = RoundedCornerShape(Dimens.RadiusBase),
                colors = CardDefaults.cardColors(containerColor = colors.accent.copy(alpha = 0.12f)),
                modifier = Modifier.weight(1f).border(1.5.dp, colors.accent, RoundedCornerShape(Dimens.RadiusBase)).padding(Dimens.SpaceSm),
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(painterResource(Res.drawable.ic_picture_as_pdf), null, tint = colors.accent, modifier = Modifier.size(20.dp))
                    Spacer(Modifier.width(8.dp))
                    Column {
                        Text("PDF View", style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Bold, color = colors.fillPrimary)
                        Text("Open PDFs with page rendering and highlights", style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary)
                    }
                }
            }
            Card(
                shape = RoundedCornerShape(Dimens.RadiusBase),
                colors = CardDefaults.cardColors(containerColor = colors.sidepane),
                modifier = Modifier.weight(1f).border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusBase)).padding(Dimens.SpaceSm),
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(painterResource(Res.drawable.ic_description), null, tint = colors.fillSecondary, modifier = Modifier.size(20.dp))
                    Spacer(Modifier.width(8.dp))
                    Column {
                        Text("Clean Text View", style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Bold, color = colors.fillPrimary)
                        Text("Distraction-free reading with typography controls", style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary)
                    }
                }
            }
        }

        HorizontalDivider(color = colors.fillQuarternary)

        Text("Default Reader Typography", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = colors.fillPrimary)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            ReaderFont.entries.forEach { font ->
                val isSelected = selectedFont == font
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .clip(RoundedCornerShape(Dimens.RadiusBase))
                        .background(if (isSelected) colors.accent else colors.sidepane)
                        .border(1.dp, if (isSelected) colors.accent else colors.border, RoundedCornerShape(Dimens.RadiusBase))
                        .clickable { onEvent(SettingsEvent.OnSetReaderFont(font)) }
                        .padding(vertical = 8.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        font.title,
                        style = MaterialTheme.typography.bodyMedium.copy(
                            fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Normal,
                            fontFamily = when (font) {
                                ReaderFont.Serif -> FontFamily.Serif
                                ReaderFont.Sans -> FontFamily.Default
                                ReaderFont.Mono -> FontFamily.Monospace
                            },
                        ),
                        color = if (isSelected) colors.onAccent else colors.fillPrimary,
                    )
                }
            }
        }

        HorizontalDivider(color = colors.fillQuarternary)

        Text("Default Reader Theme", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = colors.fillPrimary)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            ReaderTheme.entries.forEach { theme ->
                val themeCol = readerColors(theme)
                val isSelected = selectedTheme == theme
                Box(
                    modifier = Modifier
                        .size(44.dp)
                        .clip(CircleShape)
                        .background(themeCol.background)
                        .border(if (isSelected) 2.5.dp else 1.dp, if (isSelected) colors.accent else colors.border, CircleShape)
                        .clickable { onEvent(SettingsEvent.OnSetReaderTheme(theme)) },
                    contentAlignment = Alignment.Center,
                ) {
                    Text("Aa", style = MaterialTheme.typography.labelSmall.copy(fontSize = 12.sp, fontWeight = FontWeight.Bold), color = themeCol.text)
                }
            }
        }

        HorizontalDivider(color = colors.fillQuarternary)

        Text("Default Reading Font Size", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = colors.fillPrimary)
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Box(
                modifier = Modifier
                    .size(32.dp)
                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                    .background(colors.sidepane)
                    .border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusSmall))
                    .clickable { onEvent(SettingsEvent.OnSetReaderFontSize(maxOf(12, defaultFontSize - 1))) },
                contentAlignment = Alignment.Center,
            ) {
                Text("−", style = MaterialTheme.typography.titleMedium, color = colors.fillPrimary)
            }
            Text("${defaultFontSize}sp", style = MaterialTheme.typography.bodyLarge, fontWeight = FontWeight.Bold, color = colors.fillPrimary)
            Box(
                modifier = Modifier
                    .size(32.dp)
                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                    .background(colors.sidepane)
                    .border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusSmall))
                    .clickable { onEvent(SettingsEvent.OnSetReaderFontSize(minOf(28, defaultFontSize + 1))) },
                contentAlignment = Alignment.Center,
            ) {
                Text("+", style = MaterialTheme.typography.titleMedium, color = colors.fillPrimary)
            }
        }
    }
}

/** Comprehensive Keyboard Shortcuts Reference */
@Composable
private fun ShortcutsSectionView() {
    val colors = QuireTheme.colors

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceMd),
    ) {
        Text("Keyboard Shortcuts Cheat-Sheet", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = colors.fillPrimary)

        ShortcutCategoryCard(
            title = "Navigation & Windows",
            shortcuts = listOf(
                "Ctrl + K" to "Universal Command Palette & Quick Switcher",
                "Ctrl + B" to "Toggle Left Navigation Sidebar",
                "Ctrl + I" to "Toggle Right Inspector Panel",
                "Ctrl + R" to "Sync / Reload Library",
                "Ctrl + ," to "Open Preferences & Settings",
                "Esc" to "Clear Selection / Dismiss Dialog",
            ),
        )

        ShortcutCategoryCard(
            title = "Library & References",
            shortcuts = listOf(
                "Space" to "Quick Look Peek Modal on Selected Paper",
                "Ctrl + N" to "Add Reference (DOI, arXiv, BibTeX)",
                "Ctrl + A" to "Select All Items",
                "Ctrl + F / Ctrl + S" to "Search Library Items",
                "Shift + Click" to "Range Multi-Selection of Papers",
                "Ctrl + Click" to "Toggle Item Selection",
                "Delete / Backspace" to "Delete Selected Paper(s)",
                "Right-Click" to "Context Menu (items, projects, tags, saved searches)",
                "Drag to a Project" to "Add the Dragged Items to It",
                "Double-Click" to "Open Reference in Reader Tab",
            ),
        )

        ShortcutCategoryCard(
            title = "Reader & Knowledge Tools",
            shortcuts = listOf(
                "Ctrl + + / Ctrl + =" to "Zoom In (PDF) / Increase Font Size (Text)",
                "Ctrl + -" to "Zoom Out (PDF) / Decrease Font Size (Text)",
                "Ctrl + 0" to "Reset Zoom (100%) / Reset Font Size (16px)",
                "Ctrl + F" to "Toggle In-Document Search Bar",
                "Drag on PDF" to "Area Snapshot tool to extract visual quote into notes",
                "[[citeKey]]" to "Bidirectional Wikilink in notes (auto-tracks backlinks)",
                "Aa Button" to "Select Typography & Reader Palette",
                "Bottom Action Palette" to "Annotate Highlight (Yellow, Green, Blue, Purple, Red)",
            ),
        )
    }
}

@Composable
private fun ShortcutCategoryCard(title: String, shortcuts: List<Pair<String, String>>) {
    val colors = QuireTheme.colors
    Card(
        shape = RoundedCornerShape(Dimens.RadiusBase),
        colors = CardDefaults.cardColors(containerColor = colors.sidepane),
        modifier = Modifier.fillMaxWidth().border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusBase)).padding(Dimens.SpaceSm),
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text(title, style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.Bold, color = colors.accent)
            HorizontalDivider(color = colors.fillQuarternary, modifier = Modifier.padding(vertical = 2.dp))
            shortcuts.forEach { (key, desc) ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.fillMaxWidth().padding(vertical = 2.dp),
                ) {
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(Dimens.RadiusSmall))
                            .background(colors.background)
                            .border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusSmall))
                            .padding(horizontal = 8.dp, vertical = 2.dp),
                    ) {
                        Text(key, style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace, fontWeight = FontWeight.Bold), color = colors.fillPrimary)
                    }
                    Spacer(Modifier.width(12.dp))
                    Text(desc, style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary)
                }
            }
        }
    }
}

