package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledTonalButton
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
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.domain.model.SettingKind
import dev.neurokitchen.quire.presentation.components.notice.NoticeBar
import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogConfirm
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsBadge
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsGroup
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsHeader
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsItem
import dev.neurokitchen.quire.presentation.designsystem.quireColors
import dev.neurokitchen.quire.presentation.designsystem.readerColors
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.mobile.navigation.MobileSettingsPage
import dev.neurokitchen.quire.presentation.mobile.navigation.ShelfKey
import dev.neurokitchen.quire.presentation.screens.notifications.NotificationsViewModel
import dev.neurokitchen.quire.presentation.screens.settings.SettingField
import dev.neurokitchen.quire.presentation.screens.settings.SettingsEvent
import dev.neurokitchen.quire.presentation.screens.settings.SettingsSection
import dev.neurokitchen.quire.presentation.screens.settings.SettingsState
import dev.neurokitchen.quire.presentation.screens.settings.SettingsViewModel
import dev.neurokitchen.quire.presentation.screens.tools.ToolsEvent
import dev.neurokitchen.quire.presentation.screens.tools.ToolsSection
import dev.neurokitchen.quire.presentation.screens.tools.ToolsViewModel
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.destination_settings
import dev.neurokitchen.quire.resources.ic_auto_fix_high
import dev.neurokitchen.quire.resources.ic_chevron_right
import dev.neurokitchen.quire.resources.ic_create_new_folder
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_info
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_notifications
import dev.neurokitchen.quire.resources.ic_person
import dev.neurokitchen.quire.resources.ic_quire_logo
import dev.neurokitchen.quire.resources.ic_sync
import dev.neurokitchen.quire.resources.library_my_library
import dev.neurokitchen.quire.resources.library_trash
import dev.neurokitchen.quire.resources.mobile_settings_about
import dev.neurokitchen.quire.resources.mobile_settings_about_hint
import dev.neurokitchen.quire.resources.mobile_settings_account
import dev.neurokitchen.quire.resources.mobile_settings_account_email
import dev.neurokitchen.quire.resources.mobile_settings_account_email_hint
import dev.neurokitchen.quire.resources.mobile_settings_account_email_placeholder
import dev.neurokitchen.quire.resources.mobile_settings_account_hint
import dev.neurokitchen.quire.resources.mobile_settings_account_name
import dev.neurokitchen.quire.resources.mobile_settings_account_name_hint
import dev.neurokitchen.quire.resources.mobile_settings_account_name_placeholder
import dev.neurokitchen.quire.resources.mobile_settings_account_profile
import dev.neurokitchen.quire.resources.mobile_settings_app_theme
import dev.neurokitchen.quire.resources.mobile_settings_appearance
import dev.neurokitchen.quire.resources.mobile_settings_appearance_hint
import dev.neurokitchen.quire.resources.mobile_settings_libraries
import dev.neurokitchen.quire.resources.mobile_settings_libraries_empty
import dev.neurokitchen.quire.resources.mobile_settings_libraries_hint
import dev.neurokitchen.quire.resources.mobile_settings_library_default_sub
import dev.neurokitchen.quire.resources.mobile_settings_library_switch_sub
import dev.neurokitchen.quire.resources.mobile_settings_new_library_slug_hint
import dev.neurokitchen.quire.resources.mobile_settings_new_library_title
import dev.neurokitchen.quire.resources.mobile_settings_notifications_hint
import dev.neurokitchen.quire.resources.mobile_settings_notifications_unread
import dev.neurokitchen.quire.resources.mobile_settings_reader_font
import dev.neurokitchen.quire.resources.mobile_settings_reader_size
import dev.neurokitchen.quire.resources.mobile_settings_reader_theme
import dev.neurokitchen.quire.resources.mobile_settings_reading
import dev.neurokitchen.quire.resources.mobile_settings_reading_hint
import dev.neurokitchen.quire.resources.mobile_settings_sync
import dev.neurokitchen.quire.resources.mobile_settings_sync_hint
import dev.neurokitchen.quire.resources.mobile_settings_system_hint
import dev.neurokitchen.quire.resources.mobile_settings_trash_hint
import dev.neurokitchen.quire.resources.mobile_settings_version
import dev.neurokitchen.quire.resources.notifications
import dev.neurokitchen.quire.resources.settings_group_look
import dev.neurokitchen.quire.resources.settings_group_more
import dev.neurokitchen.quire.resources.settings_group_sync
import dev.neurokitchen.quire.resources.settings_group_you
import dev.neurokitchen.quire.resources.settings_header_hint
import dev.neurokitchen.quire.resources.settings_reset
import dev.neurokitchen.quire.resources.settings_save
import dev.neurokitchen.quire.resources.tools_create
import dev.neurokitchen.quire.resources.tools_default
import dev.neurokitchen.quire.resources.tools_libraries_hint
import dev.neurokitchen.quire.resources.tools_make_default
import dev.neurokitchen.quire.resources.tools_new_library
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/** The Settings tab: pages in sections, each row with its icon (Material 3 Expressive style). */
@Composable
fun SettingsTab(onOpen: (MobileSettingsPage) -> Unit) {
    val notifications by koinViewModel<NotificationsViewModel>().state.collectAsStateWithLifecycle()
    val page = @Composable { title: StringResource, hint: StringResource, icon: DrawableResource, target: MobileSettingsPage ->
        @Composable { shape: Shape ->
            SettingsItem(
                stringResource(title),
                shape,
                supporting = stringResource(hint),
                icon = icon,
                opensPage = true,
                onClick = { onOpen(target) },
                trailing = if (target == MobileSettingsPage.Notifications && notifications.unread > 0) {
                    { SettingsBadge(notifications.unread.toString()) }
                } else {
                    null
                },
            )
        }
    }
    LazyColumn(
        Modifier.fillMaxSize().background(QuireTheme.colors.background).hazeSource(LocalMobileChrome.current.haze),
        contentPadding = mobileContentPadding(topBar = false),
    ) {
        item { SettingsHeader(stringResource(Res.string.destination_settings), stringResource(Res.string.settings_header_hint)) }
        item {
            SettingsGroup(
                label = stringResource(Res.string.settings_group_look),
                rows = listOf(
                    page(Res.string.mobile_settings_appearance, Res.string.mobile_settings_appearance_hint, Res.drawable.ic_auto_fix_high, MobileSettingsPage.Appearance),
                    page(Res.string.mobile_settings_reading, Res.string.mobile_settings_reading_hint, Res.drawable.ic_description, MobileSettingsPage.Reading),
                ),
            )
        }
        item {
            SettingsGroup(
                label = stringResource(Res.string.settings_group_you),
                rows = listOf(
                    page(Res.string.mobile_settings_account, Res.string.mobile_settings_account_hint, Res.drawable.ic_person, MobileSettingsPage.Account),
                    page(Res.string.mobile_settings_libraries, Res.string.mobile_settings_libraries_hint, Res.drawable.ic_library_books, MobileSettingsPage.Libraries),
                ),
            )
        }
        item {
            SettingsGroup(
                label = stringResource(Res.string.settings_group_sync),
                rows = listOf(
                    page(Res.string.mobile_settings_sync, Res.string.mobile_settings_sync_hint, Res.drawable.ic_sync, MobileSettingsPage.Sync),
                    page(Res.string.notifications, Res.string.mobile_settings_notifications_hint, Res.drawable.ic_notifications, MobileSettingsPage.Notifications),
                ),
            )
        }
        item {
            SettingsGroup(
                label = stringResource(Res.string.settings_group_more),
                rows = listOf(
                    page(Res.string.library_trash, Res.string.mobile_settings_trash_hint, Res.drawable.ic_delete, MobileSettingsPage.Trash),
                    page(Res.string.mobile_settings_about, Res.string.mobile_settings_about_hint, Res.drawable.ic_info, MobileSettingsPage.About),
                ),
            )
        }
    }
}

/** One settings page, with its own top bar and Back. */
@Composable
fun SettingsPageScreen(
    page: MobileSettingsPage,
    onBack: () -> Unit,
    onOpenDetails: (String) -> Unit = {},
    onOpenReader: (String, String) -> Unit = { _, _ -> },
    viewModel: SettingsViewModel = koinViewModel(),
) {
    // Sync and Libraries are backed by ToolsViewModel: their own screens.
    if (page == MobileSettingsPage.Sync) return SyncPageScreen(onBack)
    if (page == MobileSettingsPage.Libraries) return LibrariesPageScreen(onBack)
    if (page == MobileSettingsPage.Notifications) return NotificationsPageScreen(onBack)
    if (page == MobileSettingsPage.Trash) {
        return ShelfScreen(
            shelf = ShelfKey(ShelfKey.Kind.Trash),
            onBack = onBack,
            onOpenDetails = onOpenDetails,
            onOpenReader = onOpenReader,
        )
    }
    val section = when (page) {
        MobileSettingsPage.Account -> SettingsSection.General
        MobileSettingsPage.Reading -> SettingsSection.Reader
        else -> SettingsSection.Appearance
    }
    LaunchedEffect(section) { viewModel.onEvent(SettingsEvent.OnSectionClick(section)) }
    val state by viewModel.state.collectAsStateWithLifecycle()
    val onEvent = viewModel::onEvent
    val chrome = LocalMobileChrome.current
    Box(Modifier.fillMaxSize().background(QuireTheme.colors.background)) {
        LazyColumn(
            Modifier.fillMaxSize().hazeSource(chrome.haze),
            contentPadding = mobileContentPadding(),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
        ) {
            when (page) {
                MobileSettingsPage.Appearance -> item { AppearancePage(state, onEvent) }
                MobileSettingsPage.Reading -> item { ReadingPage(state, onEvent) }
                MobileSettingsPage.Account -> item { AccountPage(state, onEvent) }
                MobileSettingsPage.Libraries, MobileSettingsPage.Sync, MobileSettingsPage.Notifications, MobileSettingsPage.Trash -> Unit
                MobileSettingsPage.About -> item {
                    Group {
                        Column(
                            Modifier.fillMaxWidth().padding(Dimens.SpaceLg),
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            Image(
                                painter = painterResource(Res.drawable.ic_quire_logo),
                                contentDescription = "Quire Logo",
                                modifier = Modifier.size(64.dp),
                            )
                            Spacer(Modifier.height(Dimens.SpaceMd))
                            Text("Quire", style = MaterialTheme.typography.titleLarge, color = QuireTheme.colors.fillPrimary)
                            Spacer(Modifier.height(Dimens.SpaceXs))
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
    val systemHint = stringResource(Res.string.mobile_settings_system_hint)
    SettingsGroup(
        label = stringResource(Res.string.mobile_settings_app_theme),
        rows = AppTheme.entries.map { theme ->
            @Composable { shape: Shape ->
                val palette = quireColors(theme)
                SettingsItem(
                    theme.title,
                    shape,
                    supporting = if (theme == AppTheme.System) systemHint else null,
                    // The palette: its background with its accent and text colours.
                    leading = {
                        Row(
                            Modifier.clip(CircleShape).background(palette.background).border(1.dp, QuireTheme.colors.paneDivider, CircleShape).padding(horizontal = 10.dp, vertical = 8.dp),
                            horizontalArrangement = Arrangement.spacedBy(6.dp),
                        ) {
                            Box(Modifier.size(12.dp).clip(CircleShape).background(palette.accent))
                            Box(Modifier.size(12.dp).clip(CircleShape).background(palette.fillPrimary))
                        }
                    },
                    onClick = { onEvent(SettingsEvent.OnSetAppTheme(theme)) },
                    trailing = { RadioButton(selected = theme == state.appTheme, onClick = { onEvent(SettingsEvent.OnSetAppTheme(theme)) }) },
                )
            }
        },
    )
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

/**
 * Settings › Account on the phone: the person's name and lookup email in M3 Expressive cards.
 * Desktop-only folder paths (`access.downloads`) are excluded on mobile.
 */
@Composable
private fun AccountPage(state: SettingsState, onEvent: (SettingsEvent) -> Unit) {
    val fields = state.fields
        .filterNot { it.key.contains('*') || it.kind == SettingKind.Path }
        .sortedBy {
            when (it.key) {
                "user.name" -> 0
                "user.email" -> 1
                else -> 2
            }
        }
    if (fields.isEmpty()) return
    SettingsGroup(
        label = stringResource(Res.string.mobile_settings_account_profile),
        rows = fields.map { field ->
            @Composable { shape: Shape ->
                Box(Modifier.fillMaxWidth().clip(shape).background(QuireTheme.colors.cardBackground)) {
                    AccountFieldSegment(field, onEvent)
                }
            }
        },
    )
}

@Composable
private fun AccountFieldSegment(field: SettingField, onEvent: (SettingsEvent) -> Unit) {
    val colors = QuireTheme.colors
    val (title, supporting, placeholder, icon) = when (field.key) {
        "user.name" -> AccountFieldMeta(
            title = stringResource(Res.string.mobile_settings_account_name),
            supporting = stringResource(Res.string.mobile_settings_account_name_hint),
            placeholder = stringResource(Res.string.mobile_settings_account_name_placeholder),
            icon = Res.drawable.ic_person,
        )
        "user.email" -> AccountFieldMeta(
            title = stringResource(Res.string.mobile_settings_account_email),
            supporting = stringResource(Res.string.mobile_settings_account_email_hint),
            placeholder = stringResource(Res.string.mobile_settings_account_email_placeholder),
            icon = Res.drawable.ic_description,
        )
        else -> AccountFieldMeta(
            title = field.help.replaceFirstChar { it.uppercase() },
            supporting = null,
            placeholder = "",
            icon = Res.drawable.ic_person,
        )
    }
    Column(Modifier.fillMaxWidth().padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceMd)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                Modifier.size(40.dp).clip(CircleShape).background(colors.accent.copy(alpha = 0.16f)),
                contentAlignment = Alignment.Center,
            ) {
                Icon(painterResource(icon), contentDescription = null, tint = colors.accent, modifier = Modifier.size(22.dp))
            }
            Spacer(Modifier.width(Dimens.SpaceMd))
            Column(Modifier.weight(1f)) {
                Text(title, style = MaterialTheme.typography.titleMedium, color = colors.fillPrimary)
                if (supporting != null) {
                    Text(supporting, style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary)
                }
            }
            if (field.isDirty) {
                Spacer(Modifier.width(Dimens.SpaceXs))
                FilledTonalButton(
                    onClick = { onEvent(SettingsEvent.OnApply(field.key)) },
                    shape = RoundedCornerShape(Dimens.RadiusFull),
                    colors = ButtonDefaults.filledTonalButtonColors(
                        containerColor = colors.accent.copy(alpha = 0.16f),
                        contentColor = colors.accent,
                    ),
                    contentPadding = PaddingValues(horizontal = Dimens.SpaceSm, vertical = Dimens.SpaceMin),
                ) {
                    Text(stringResource(Res.string.settings_save), style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.SemiBold)
                }
            } else if (!field.saved.isNullOrBlank()) {
                Spacer(Modifier.width(Dimens.SpaceXs))
                TextButton(
                    onClick = { onEvent(SettingsEvent.OnReset(field.key)) },
                    contentPadding = PaddingValues(horizontal = Dimens.SpaceXs, vertical = Dimens.SpaceMin),
                ) {
                    Text(stringResource(Res.string.settings_reset), style = MaterialTheme.typography.labelMedium, color = colors.fillSecondary)
                }
            }
        }
        Spacer(Modifier.height(Dimens.SpaceSm))
        OutlinedTextField(
            value = field.draft,
            onValueChange = { onEvent(SettingsEvent.OnDraftChange(field.key, it)) },
            placeholder = if (placeholder.isNotEmpty()) {
                { Text(placeholder, color = colors.fillTertiary) }
            } else {
                null
            },
            isError = field.error != null,
            supportingText = field.error?.let { { Text(it) } },
            singleLine = true,
            shape = RoundedCornerShape(Dimens.RadiusBase),
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

private data class AccountFieldMeta(
    val title: String,
    val supporting: String?,
    val placeholder: String,
    val icon: DrawableResource,
)

/**
 * Settings › Libraries on the phone: lists the known libraries as Material 3 Expressive rows
 * (tapping opens a library, trailing pill marks the default or makes another the default),
 * and lets the user create a new library via a clean dialog.
 */
@Composable
fun LibrariesPageScreen(onBack: () -> Unit, viewModel: ToolsViewModel = koinViewModel()) {
    LaunchedEffect(Unit) { viewModel.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Libraries)) }
    val state by viewModel.state.collectAsStateWithLifecycle()
    val onEvent = viewModel::onEvent
    val chrome = LocalMobileChrome.current
    val colors = QuireTheme.colors
    var showCreateDialog by remember { mutableStateOf(false) }

    Box(Modifier.fillMaxSize().background(colors.background)) {
        LazyColumn(
            Modifier.fillMaxSize().hazeSource(chrome.haze),
            contentPadding = mobileContentPadding(),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
        ) {
            item {
                val myLibraryLabel = stringResource(Res.string.library_my_library)
                val defaultSub = stringResource(Res.string.mobile_settings_library_default_sub)
                val switchSub = stringResource(Res.string.mobile_settings_library_switch_sub)
                val emptyHint = stringResource(Res.string.mobile_settings_libraries_empty)
                val rows: List<@Composable (Shape) -> Unit> = if (state.libraries.isNotEmpty()) {
                    state.libraries.map { library ->
                        @Composable { shape: Shape ->
                            val displayTitle = when (val n = library.name) {
                                null, "default" -> myLibraryLabel
                                else -> n
                            }
                            SettingsItem(
                                title = displayTitle,
                                shape = shape,
                                supporting = if (library.isDefault) defaultSub else switchSub,
                                icon = Res.drawable.ic_library_books,
                                enabled = !state.isWorking,
                                onClick = { onEvent(ToolsEvent.OnOpenLibrary(library.name)) },
                                trailing = {
                                    if (library.isDefault) {
                                        Text(
                                            text = stringResource(Res.string.tools_default),
                                            style = MaterialTheme.typography.labelMedium,
                                            fontWeight = FontWeight.SemiBold,
                                            color = colors.accent,
                                            modifier = Modifier
                                                .clip(CircleShape)
                                                .background(colors.accent.copy(alpha = 0.16f))
                                                .padding(horizontal = 10.dp, vertical = 4.dp),
                                        )
                                    } else if (library.name != null) {
                                        TextButton(
                                            onClick = { onEvent(ToolsEvent.OnSetDefaultLibrary(library.name)) },
                                            enabled = !state.isWorking,
                                        ) {
                                            Text(
                                                text = stringResource(Res.string.tools_make_default),
                                                style = MaterialTheme.typography.labelMedium,
                                                fontWeight = FontWeight.SemiBold,
                                                color = colors.accent,
                                            )
                                        }
                                    }
                                },
                            )
                        }
                    }
                } else {
                    listOf { shape: Shape ->
                        SettingsItem(
                            title = myLibraryLabel,
                            shape = shape,
                            supporting = emptyHint,
                            icon = Res.drawable.ic_library_books,
                            trailing = {
                                Text(
                                    text = stringResource(Res.string.tools_default),
                                    style = MaterialTheme.typography.labelMedium,
                                    fontWeight = FontWeight.SemiBold,
                                    color = colors.accent,
                                    modifier = Modifier
                                        .clip(CircleShape)
                                        .background(colors.accent.copy(alpha = 0.16f))
                                        .padding(horizontal = 10.dp, vertical = 4.dp),
                                )
                            },
                        )
                    }
                }
                SettingsGroup(
                    label = stringResource(Res.string.mobile_settings_libraries),
                    rows = rows,
                )
            }
            item {
                SettingsGroup(
                    rows = listOf { shape: Shape ->
                        SettingsItem(
                            title = stringResource(Res.string.mobile_settings_new_library_title),
                            shape = shape,
                            supporting = stringResource(Res.string.tools_libraries_hint),
                            icon = Res.drawable.ic_create_new_folder,
                            enabled = !state.isWorking,
                            opensPage = true,
                            onClick = { showCreateDialog = true },
                        )
                    },
                )
            }
        }
        MobileTopBar(stringResource(Res.string.mobile_settings_libraries), onBack, chrome.haze)
        state.notice?.let { notice ->
            NoticeBar(
                notice = notice,
                onDismiss = { onEvent(ToolsEvent.OnDismissNotice) },
                modifier = Modifier.align(Alignment.BottomCenter).padding(Dimens.SpaceMd),
            )
        }
    }

    if (showCreateDialog) {
        NewLibraryDialog(
            working = state.isWorking,
            onDismiss = { showCreateDialog = false },
            onCreate = { name ->
                showCreateDialog = false
                onEvent(ToolsEvent.OnCreateLibrary(name))
            },
        )
    }
}

@Composable
private fun NewLibraryDialog(
    working: Boolean,
    onDismiss: () -> Unit,
    onCreate: (String) -> Unit,
) {
    var name by remember { mutableStateOf("") }
    DialogShell(
        title = stringResource(Res.string.tools_new_library),
        subtitle = stringResource(Res.string.mobile_settings_new_library_slug_hint),
        icon = Res.drawable.ic_create_new_folder,
        width = 360.dp,
        onDismiss = onDismiss,
        actions = {
            DialogCancel(onDismiss)
            DialogConfirm(
                label = stringResource(Res.string.tools_create),
                onClick = { if (name.isNotBlank()) onCreate(name.trim()) },
                enabled = name.isNotBlank(),
                working = working,
            )
        },
    ) {
        OutlinedTextField(
            value = name,
            onValueChange = { name = it },
            placeholder = { Text("lab", color = QuireTheme.colors.fillTertiary) },
            singleLine = true,
            shape = RoundedCornerShape(Dimens.RadiusBase),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { if (name.isNotBlank()) onCreate(name.trim()) }),
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

private fun MobileSettingsPage.title() = when (this) {
    MobileSettingsPage.Appearance -> Res.string.mobile_settings_appearance
    MobileSettingsPage.Reading -> Res.string.mobile_settings_reading
    MobileSettingsPage.Account -> Res.string.mobile_settings_account
    MobileSettingsPage.Libraries -> Res.string.mobile_settings_libraries
    MobileSettingsPage.Sync -> Res.string.mobile_settings_sync
    MobileSettingsPage.Notifications -> Res.string.notifications
    MobileSettingsPage.About -> Res.string.mobile_settings_about
    MobileSettingsPage.Trash -> Res.string.library_trash
}
