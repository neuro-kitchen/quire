package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsBadge
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.mobile.navigation.MobileSettingsPage
import dev.neurokitchen.quire.presentation.mobile.navigation.ShelfKey
import dev.neurokitchen.quire.presentation.screens.notifications.NotificationsViewModel
import dev.neurokitchen.quire.presentation.screens.settings.SettingsEvent
import dev.neurokitchen.quire.presentation.screens.settings.SettingsSection
import dev.neurokitchen.quire.presentation.screens.settings.SettingsViewModel
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.destination_settings
import dev.neurokitchen.quire.resources.ic_auto_fix_high
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_info
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_notifications
import dev.neurokitchen.quire.resources.ic_person
import dev.neurokitchen.quire.resources.ic_qr_code
import dev.neurokitchen.quire.resources.ic_quire_logo
import dev.neurokitchen.quire.resources.ic_sync
import dev.neurokitchen.quire.resources.library_trash
import dev.neurokitchen.quire.resources.mobile_settings_about
import dev.neurokitchen.quire.resources.mobile_settings_about_hint
import dev.neurokitchen.quire.resources.mobile_settings_account
import dev.neurokitchen.quire.resources.mobile_settings_account_hint
import dev.neurokitchen.quire.resources.mobile_settings_libraries
import dev.neurokitchen.quire.resources.mobile_settings_libraries_hint
import dev.neurokitchen.quire.resources.mobile_settings_notifications_hint
import dev.neurokitchen.quire.resources.mobile_settings_sync
import dev.neurokitchen.quire.resources.mobile_settings_sync_hint
import dev.neurokitchen.quire.resources.mobile_settings_trash_hint
import dev.neurokitchen.quire.resources.mobile_settings_version
import dev.neurokitchen.quire.resources.notifications
import dev.neurokitchen.quire.resources.settings_appearance_reading
import dev.neurokitchen.quire.resources.settings_appearance_reading_sub
import dev.neurokitchen.quire.resources.settings_profile_sub_unset
import dev.neurokitchen.quire.resources.settings_profile_unset
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/**
 * WhatsApp-inspired Settings tab:
 * - Prominent top Profile identity row (Avatar/Initials + Name + Email + quick QR/Sync button)
 * - Full-width subtle hairline divider
 * - Clean, unboxed single-column list of settings items (monochrome/subtle icons, no heavy boxes or chevrons)
 * - Quiet centered brand/version footer at the bottom
 */
@Composable
fun SettingsTab(
    onOpen: (MobileSettingsPage) -> Unit,
    settingsViewModel: SettingsViewModel = koinViewModel(),
) {
    LaunchedEffect(Unit) {
        settingsViewModel.onEvent(SettingsEvent.OnSectionClick(SettingsSection.General))
    }
    val state by settingsViewModel.state.collectAsStateWithLifecycle()
    val notifications by koinViewModel<NotificationsViewModel>().state.collectAsStateWithLifecycle()
    val colors = QuireTheme.colors

    val savedName = state.fields.firstOrNull { it.key == "user.name" }?.saved?.takeIf { it.isNotBlank() }
    val savedEmail = state.fields.firstOrNull { it.key == "user.email" }?.saved?.takeIf { it.isNotBlank() }

    LazyColumn(
        Modifier
            .fillMaxSize()
            .background(colors.background)
            .hazeSource(LocalMobileChrome.current.haze),
        contentPadding = mobileContentPadding(topBar = false),
    ) {
        item {
            LargeTitle(
                title = stringResource(Res.string.destination_settings),
                subtitle = null,
            )
        }
        item {
            ProfileHeaderRow(
                name = savedName,
                email = savedEmail,
                onClickProfile = { onOpen(MobileSettingsPage.Account) },
                onClickQrSync = { onOpen(MobileSettingsPage.Sync) },
            )
        }
        item {
            HorizontalDivider(
                color = colors.border.copy(alpha = 0.35f),
                modifier = Modifier.padding(vertical = 4.dp),
            )
        }
        item {
            WhatsAppSettingsRow(
                icon = Res.drawable.ic_person,
                title = stringResource(Res.string.mobile_settings_account),
                subtitle = stringResource(Res.string.mobile_settings_account_hint),
                onClick = { onOpen(MobileSettingsPage.Account) },
            )
        }
        item {
            WhatsAppSettingsRow(
                icon = Res.drawable.ic_auto_fix_high,
                title = stringResource(Res.string.settings_appearance_reading),
                subtitle = stringResource(
                    Res.string.settings_appearance_reading_sub,
                    state.appTheme.title,
                    state.readerTheme.title,
                    state.readerFont.title,
                    state.readerFontSize,
                ),
                onClick = { onOpen(MobileSettingsPage.Appearance) },
            )
        }
        item {
            WhatsAppSettingsRow(
                icon = Res.drawable.ic_library_books,
                title = stringResource(Res.string.mobile_settings_libraries),
                subtitle = stringResource(Res.string.mobile_settings_libraries_hint),
                onClick = { onOpen(MobileSettingsPage.Libraries) },
            )
        }
        item {
            WhatsAppSettingsRow(
                icon = Res.drawable.ic_sync,
                title = stringResource(Res.string.mobile_settings_sync),
                subtitle = stringResource(Res.string.mobile_settings_sync_hint),
                onClick = { onOpen(MobileSettingsPage.Sync) },
            )
        }
        item {
            WhatsAppSettingsRow(
                icon = Res.drawable.ic_notifications,
                title = stringResource(Res.string.notifications),
                subtitle = stringResource(Res.string.mobile_settings_notifications_hint),
                onClick = { onOpen(MobileSettingsPage.Notifications) },
                trailing = if (notifications.unread > 0) {
                    { SettingsBadge(notifications.unread.toString()) }
                } else {
                    null
                },
            )
        }
        item {
            WhatsAppSettingsRow(
                icon = Res.drawable.ic_delete,
                title = stringResource(Res.string.library_trash),
                subtitle = stringResource(Res.string.mobile_settings_trash_hint),
                onClick = { onOpen(MobileSettingsPage.Trash) },
            )
        }
        item {
            WhatsAppSettingsRow(
                icon = Res.drawable.ic_info,
                title = stringResource(Res.string.mobile_settings_about),
                subtitle = stringResource(Res.string.mobile_settings_about_hint),
                onClick = { onOpen(MobileSettingsPage.About) },
            )
        }
        item {
            SettingsBrandFooter(
                version = state.version,
                onClick = { onOpen(MobileSettingsPage.About) },
            )
        }
    }
}

@Composable
private fun ProfileHeaderRow(
    name: String?,
    email: String?,
    onClickProfile: () -> Unit,
    onClickQrSync: () -> Unit,
) {
    val colors = QuireTheme.colors
    val initials = remember(name) {
        name?.trim()
            ?.split(Regex("\\s+"))
            ?.filter { it.isNotEmpty() }
            ?.take(2)
            ?.joinToString("") { it.first().uppercase() }
            .orEmpty()
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClickProfile)
            .padding(horizontal = 20.dp, vertical = 14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .size(60.dp)
                .clip(CircleShape)
                .background(colors.accent.copy(alpha = 0.16f)),
            contentAlignment = Alignment.Center,
        ) {
            if (initials.isNotEmpty()) {
                Text(
                    text = initials,
                    style = MaterialTheme.typography.titleLarge.copy(
                        fontWeight = FontWeight.Bold,
                        fontSize = 21.sp,
                    ),
                    color = colors.accent,
                )
            } else {
                Icon(
                    painter = painterResource(Res.drawable.ic_person),
                    contentDescription = null,
                    tint = colors.accent,
                    modifier = Modifier.size(30.dp),
                )
            }
        }

        Spacer(Modifier.width(16.dp))

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = name ?: stringResource(Res.string.settings_profile_unset),
                style = MaterialTheme.typography.titleMedium.copy(
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 18.sp,
                ),
                color = colors.fillPrimary,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(Modifier.height(2.dp))
            Text(
                text = email ?: stringResource(Res.string.settings_profile_sub_unset),
                style = MaterialTheme.typography.bodyMedium.copy(fontSize = 13.sp),
                color = colors.fillSecondary,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }

        Spacer(Modifier.width(12.dp))

        Box(
            modifier = Modifier
                .size(40.dp)
                .clip(CircleShape)
                .background(colors.fillQuinary)
                .clickable(onClick = onClickQrSync),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                painter = painterResource(Res.drawable.ic_qr_code),
                contentDescription = stringResource(Res.string.mobile_settings_sync),
                tint = colors.accent,
                modifier = Modifier.size(20.dp),
            )
        }
    }
}

@Composable
internal fun WhatsAppSettingsRow(
    icon: DrawableResource,
    title: String,
    subtitle: String?,
    onClick: () -> Unit,
    trailing: (@Composable () -> Unit)? = null,
) {
    val colors = QuireTheme.colors
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 24.dp, vertical = 15.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            painter = painterResource(icon),
            contentDescription = null,
            tint = colors.fillSecondary,
            modifier = Modifier.size(24.dp),
        )
        Spacer(Modifier.width(22.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge.copy(
                    fontWeight = FontWeight.Normal,
                    fontSize = 16.sp,
                ),
                color = colors.fillPrimary,
            )
            if (!subtitle.isNullOrBlank()) {
                Spacer(Modifier.height(2.dp))
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.bodyMedium.copy(fontSize = 13.sp),
                    color = colors.fillSecondary,
                )
            }
        }
        if (trailing != null) {
            Spacer(Modifier.width(12.dp))
            trailing()
        }
    }
}

@Composable
internal fun WhatsAppSectionTitle(title: String) {
    Text(
        text = title,
        style = MaterialTheme.typography.labelLarge.copy(
            fontWeight = FontWeight.SemiBold,
            fontSize = 13.sp,
        ),
        color = QuireTheme.colors.fillSecondary,
        modifier = Modifier.padding(start = 24.dp, end = 24.dp, top = 12.dp, bottom = 4.dp),
    )
}

@Composable
internal fun SettingsBrandFooter(
    version: String,
    onClick: (() -> Unit)? = null,
) {
    val colors = QuireTheme.colors
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier)
            .padding(vertical = 28.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Image(
                painter = painterResource(Res.drawable.ic_quire_logo),
                contentDescription = null,
                modifier = Modifier.size(18.dp),
            )
            Text(
                text = "Quire",
                style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.Bold),
                color = colors.fillSecondary,
            )
        }
        if (version.isNotBlank()) {
            Text(
                text = stringResource(Res.string.mobile_settings_version, version),
                style = MaterialTheme.typography.labelSmall,
                color = colors.fillTertiary,
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
                MobileSettingsPage.Appearance, MobileSettingsPage.Reading -> item {
                    AppearanceAndReadingPage(state, onEvent)
                }
                MobileSettingsPage.Account -> item {
                    AccountPage(state, onEvent)
                }
                MobileSettingsPage.Libraries, MobileSettingsPage.Sync, MobileSettingsPage.Notifications, MobileSettingsPage.Trash -> Unit
                MobileSettingsPage.About -> item {
                    AboutAndLegalPage(version = state.version)
                }
            }
        }
        MobileTopBar(stringResource(page.title()), onBack, chrome.haze)
    }
}

private fun MobileSettingsPage.title() = when (this) {
    MobileSettingsPage.Appearance -> Res.string.settings_appearance_reading
    MobileSettingsPage.Reading -> Res.string.settings_appearance_reading
    MobileSettingsPage.Account -> Res.string.mobile_settings_account
    MobileSettingsPage.Libraries -> Res.string.mobile_settings_libraries
    MobileSettingsPage.Sync -> Res.string.mobile_settings_sync
    MobileSettingsPage.Notifications -> Res.string.notifications
    MobileSettingsPage.About -> Res.string.mobile_settings_about
    MobileSettingsPage.Trash -> Res.string.library_trash
}
