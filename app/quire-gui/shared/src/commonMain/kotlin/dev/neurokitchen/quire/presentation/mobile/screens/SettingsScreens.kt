package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.domain.model.SettingKind
import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import dev.neurokitchen.quire.presentation.designsystem.quireColors
import dev.neurokitchen.quire.presentation.designsystem.readerColors
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.mobile.navigation.MobileSettingsPage
import dev.neurokitchen.quire.presentation.screens.settings.SettingField
import dev.neurokitchen.quire.presentation.screens.settings.SettingsEvent
import dev.neurokitchen.quire.presentation.screens.settings.SettingsSection
import dev.neurokitchen.quire.presentation.screens.settings.SettingsState
import dev.neurokitchen.quire.presentation.screens.settings.SettingsViewModel
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.destination_settings
import dev.neurokitchen.quire.resources.ic_chevron_right
import dev.neurokitchen.quire.resources.mobile_settings_about
import dev.neurokitchen.quire.resources.mobile_settings_about_hint
import dev.neurokitchen.quire.resources.mobile_settings_account
import dev.neurokitchen.quire.resources.mobile_settings_account_hint
import dev.neurokitchen.quire.resources.mobile_settings_app_theme
import dev.neurokitchen.quire.resources.mobile_settings_appearance
import dev.neurokitchen.quire.resources.mobile_settings_appearance_hint
import dev.neurokitchen.quire.resources.mobile_settings_libraries
import dev.neurokitchen.quire.resources.mobile_settings_libraries_hint
import dev.neurokitchen.quire.resources.mobile_settings_libraries_empty
import dev.neurokitchen.quire.resources.mobile_settings_reading
import dev.neurokitchen.quire.resources.mobile_settings_reading_hint
import dev.neurokitchen.quire.resources.mobile_settings_reader_font
import dev.neurokitchen.quire.resources.mobile_settings_reader_size
import dev.neurokitchen.quire.resources.mobile_settings_reader_theme
import dev.neurokitchen.quire.resources.mobile_settings_system_hint
import dev.neurokitchen.quire.resources.mobile_settings_version
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/** The Settings tab: a list of pages, as Android's own settings. */
@Composable
fun SettingsTab(onOpen: (MobileSettingsPage) -> Unit) {
    LazyColumn(
        Modifier.fillMaxSize().hazeSource(LocalMobileChrome.current.haze),
        contentPadding = mobileContentPadding(topBar = false),
    ) {
        item { LargeTitle(stringResource(Res.string.destination_settings), null) }
        item {
            Group {
                PageRow(Res.string.mobile_settings_appearance, Res.string.mobile_settings_appearance_hint) { onOpen(MobileSettingsPage.Appearance) }
                PageRow(Res.string.mobile_settings_reading, Res.string.mobile_settings_reading_hint) { onOpen(MobileSettingsPage.Reading) }
            }
        }
        item { Spacer(Modifier.size(Dimens.SpaceMd)) }
        item {
            Group {
                PageRow(Res.string.mobile_settings_account, Res.string.mobile_settings_account_hint) { onOpen(MobileSettingsPage.Account) }
                PageRow(Res.string.mobile_settings_libraries, Res.string.mobile_settings_libraries_hint) { onOpen(MobileSettingsPage.Libraries) }
            }
        }
        item { Spacer(Modifier.size(Dimens.SpaceMd)) }
        item {
            Group { PageRow(Res.string.mobile_settings_about, Res.string.mobile_settings_about_hint) { onOpen(MobileSettingsPage.About) } }
        }
    }
}

/** One settings page, with its own top bar and Back. */
@Composable
fun SettingsPageScreen(
    page: MobileSettingsPage,
    onBack: () -> Unit,
    viewModel: SettingsViewModel = koinViewModel(),
) {
    val section = when (page) {
        MobileSettingsPage.Account -> SettingsSection.General
        MobileSettingsPage.Libraries -> SettingsSection.Libraries
        MobileSettingsPage.Reading -> SettingsSection.Reader
        else -> SettingsSection.Appearance
    }
    LaunchedEffect(section) { viewModel.onEvent(SettingsEvent.OnSectionClick(section)) }
    val state by viewModel.state.collectAsStateWithLifecycle()
    val onEvent = viewModel::onEvent
    val chrome = LocalMobileChrome.current
    Box(Modifier.fillMaxSize()) {
        LazyColumn(
            Modifier.fillMaxSize().hazeSource(chrome.haze),
            contentPadding = mobileContentPadding(),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
        ) {
            when (page) {
                MobileSettingsPage.Appearance -> item { AppearancePage(state, onEvent) }
                MobileSettingsPage.Reading -> item { ReadingPage(state, onEvent) }
                MobileSettingsPage.Account -> item { FieldsPage(state, onEvent, emptyNote = null) }
                MobileSettingsPage.Libraries -> item { FieldsPage(state, onEvent, emptyNote = stringResource(Res.string.mobile_settings_libraries_empty)) }
                MobileSettingsPage.About -> item {
                    Group {
                        Column(Modifier.fillMaxWidth().padding(Dimens.SpaceMd)) {
                            Text("Quire", style = MaterialTheme.typography.titleMedium, color = QuireTheme.colors.fillPrimary)
                            Text(stringResource(Res.string.mobile_settings_version, state.version), style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary)
                        }
                    }
                }
            }
        }
        MobileTopBar(stringResource(page.title()), onBack, chrome.haze)
    }
}

/** The app's theme: System (follows the device) and the five palettes, each previewed. */
@Composable
private fun AppearancePage(state: SettingsState, onEvent: (SettingsEvent) -> Unit) {
    GroupLabel(stringResource(Res.string.mobile_settings_app_theme))
    Group {
        AppTheme.entries.forEach { theme ->
            val palette = quireColors(theme)
            Row(
                Modifier
                    .fillMaxWidth()
                    .heightIn(min = 64.dp)
                    .clickable { onEvent(SettingsEvent.OnSetAppTheme(theme)) }
                    .padding(horizontal = Dimens.SpaceMd),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
            ) {
                RadioButton(selected = theme == state.appTheme, onClick = { onEvent(SettingsEvent.OnSetAppTheme(theme)) })
                Column(Modifier.weight(1f)) {
                    Text(theme.title, style = MaterialTheme.typography.bodyLarge, color = QuireTheme.colors.fillPrimary)
                    if (theme == AppTheme.System) {
                        Text(stringResource(Res.string.mobile_settings_system_hint), style = MaterialTheme.typography.bodySmall, color = QuireTheme.colors.fillSecondary)
                    }
                }
                // The palette: its background with its accent and text colours.
                Row(
                    Modifier.clip(CircleShape).background(palette.background).border(1.dp, QuireTheme.colors.paneDivider, CircleShape).padding(horizontal = 10.dp, vertical = 6.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    Box(Modifier.size(12.dp).clip(CircleShape).background(palette.accent))
                    Box(Modifier.size(12.dp).clip(CircleShape).background(palette.fillPrimary))
                }
            }
        }
    }
}

/** How papers open: page colours, typeface, text size (the reader's Aa changes them per paper). */
@Composable
private fun ReadingPage(state: SettingsState, onEvent: (SettingsEvent) -> Unit) {
    val colors = QuireTheme.colors
    GroupLabel(stringResource(Res.string.mobile_settings_reader_theme))
    Row(Modifier.padding(horizontal = Dimens.SpaceMd), horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceSm)) {
        ReaderTheme.entries.forEach { theme ->
            val selected = theme == state.readerTheme
            val swatch = readerColors(theme)
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Box(
                    Modifier
                        .size(48.dp)
                        .clip(CircleShape)
                        .background(swatch.background)
                        .border(if (selected) 3.dp else 1.dp, if (selected) colors.accent else colors.paneDivider, CircleShape)
                        .clickable(onClickLabel = theme.title) { onEvent(SettingsEvent.OnSetReaderTheme(theme)) },
                    contentAlignment = Alignment.Center,
                ) { Text("Aa", color = swatch.text, style = MaterialTheme.typography.labelLarge) }
            }
        }
    }
    Text(state.readerTheme.title, style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary, modifier = Modifier.padding(start = Dimens.SpaceLg, top = Dimens.SpaceMin))
    GroupLabel(stringResource(Res.string.mobile_settings_reader_font))
    SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth().padding(horizontal = Dimens.SpaceMd)) {
        ReaderFont.entries.forEachIndexed { i, font ->
            SegmentedButton(
                selected = font == state.readerFont,
                onClick = { onEvent(SettingsEvent.OnSetReaderFont(font)) },
                shape = SegmentedButtonDefaults.itemShape(i, ReaderFont.entries.size),
            ) {
                Text(font.title, fontFamily = when (font) {
                    ReaderFont.Serif -> FontFamily.Serif
                    ReaderFont.Sans -> FontFamily.SansSerif
                    ReaderFont.Mono -> FontFamily.Monospace
                })
            }
        }
    }
    GroupLabel(stringResource(Res.string.mobile_settings_reader_size) + " · ${state.readerFontSize}")
    Slider(
        value = state.readerFontSize.toFloat(),
        onValueChange = { onEvent(SettingsEvent.OnSetReaderFontSize(it.toInt())) },
        valueRange = 12f..28f,
        steps = 15,
        modifier = Modifier.padding(horizontal = Dimens.SpaceLg),
    )
}

/** A form section's settings, labelled in words (no config keys); text saved on Done or leaving. */
@Composable
private fun FieldsPage(state: SettingsState, onEvent: (SettingsEvent) -> Unit, emptyNote: String?) {
    // Settings that are other settings' sub-tables (export profiles, libraries by name) stay on the desktop.
    val fields = state.fields.filterNot { it.key.contains('*') }
    if (fields.isEmpty() && emptyNote != null) {
        EmptyNote(emptyNote)
        return
    }
    Group {
        fields.forEach { field -> FieldRow(field, onEvent) }
    }
}

@Composable
private fun FieldRow(field: SettingField, onEvent: (SettingsEvent) -> Unit) {
    val colors = QuireTheme.colors
    val label = field.help.replaceFirstChar { it.uppercase() }
    Column(Modifier.fillMaxWidth().padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceSm)) {
        when (field.kind) {
            SettingKind.Bool -> Row(verticalAlignment = Alignment.CenterVertically) {
                Text(label, style = MaterialTheme.typography.bodyLarge, color = colors.fillPrimary, modifier = Modifier.weight(1f))
                Switch(checked = field.draft == "true", onCheckedChange = { onEvent(SettingsEvent.OnDraftChange(field.key, it.toString())) })
            }
            SettingKind.Choice -> {
                Text(label, style = MaterialTheme.typography.bodyLarge, color = colors.fillPrimary)
                Spacer(Modifier.size(Dimens.SpaceXs))
                SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
                    field.choices.forEachIndexed { i, choice ->
                        SegmentedButton(
                            selected = field.draft == choice,
                            onClick = { onEvent(SettingsEvent.OnDraftChange(field.key, choice)) },
                            shape = SegmentedButtonDefaults.itemShape(i, field.choices.size),
                        ) { Text(choice) }
                    }
                }
            }
            else -> OutlinedTextField(
                value = field.draft,
                onValueChange = { onEvent(SettingsEvent.OnDraftChange(field.key, it)) },
                label = { Text(label) },
                isError = field.error != null,
                supportingText = field.error?.let { { Text(it) } },
                singleLine = true,
                keyboardOptions = KeyboardOptions(
                    keyboardType = when (field.kind) {
                        SettingKind.Email -> KeyboardType.Email
                        SettingKind.Number -> KeyboardType.Number
                        else -> KeyboardType.Text
                    },
                    imeAction = ImeAction.Done,
                ),
                keyboardActions = KeyboardActions(onDone = { onEvent(SettingsEvent.OnApply(field.key)) }),
                modifier = Modifier
                    .fillMaxWidth()
                    .onFocusChanged { if (!it.isFocused && field.isDirty) onEvent(SettingsEvent.OnApply(field.key)) },
            )
        }
    }
}

/** A row opening a settings page: its name and what is in it. */
@Composable
private fun PageRow(title: StringResource, hint: StringResource, onClick: () -> Unit) {
    val colors = QuireTheme.colors
    Row(
        Modifier.fillMaxWidth().heightIn(min = 64.dp).clickable(onClick = onClick).padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceXs),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text(stringResource(title), style = MaterialTheme.typography.bodyLarge, color = colors.fillPrimary)
            Text(stringResource(hint), style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary)
        }
        Spacer(Modifier.width(Dimens.SpaceXs))
        Icon(painterResource(Res.drawable.ic_chevron_right), contentDescription = null, tint = colors.fillSecondary)
    }
}

private fun MobileSettingsPage.title() = when (this) {
    MobileSettingsPage.Appearance -> Res.string.mobile_settings_appearance
    MobileSettingsPage.Reading -> Res.string.mobile_settings_reading
    MobileSettingsPage.Account -> Res.string.mobile_settings_account
    MobileSettingsPage.Libraries -> Res.string.mobile_settings_libraries
    MobileSettingsPage.About -> Res.string.mobile_settings_about
}
