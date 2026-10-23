package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.presentation.designsystem.quireColors
import dev.neurokitchen.quire.presentation.designsystem.readerColors
import dev.neurokitchen.quire.presentation.screens.settings.SettingsEvent
import dev.neurokitchen.quire.presentation.screens.settings.SettingsState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_auto_fix_high
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.mobile_settings_app_theme
import dev.neurokitchen.quire.resources.mobile_settings_reader_font
import dev.neurokitchen.quire.resources.mobile_settings_reader_size
import dev.neurokitchen.quire.resources.mobile_settings_reader_theme
import dev.neurokitchen.quire.resources.mobile_settings_reading
import dev.neurokitchen.quire.resources.mobile_settings_system_hint
import dev.neurokitchen.quire.resources.settings_choose_font
import dev.neurokitchen.quire.resources.settings_choose_theme
import dev.neurokitchen.quire.resources.settings_display_section
import dev.neurokitchen.quire.resources.settings_reader_preview_sample
import org.jetbrains.compose.resources.stringResource

/**
 * WhatsApp "Chats / Display" style combined Appearance & Reading page:
 * - Display section with a single "Theme" row that opens a radio dialog
 * - Reading section with a live sample preview card, page colour swatches, typeface dialog row, and size slider
 */
@Composable
internal fun AppearanceAndReadingPage(
    state: SettingsState,
    onEvent: (SettingsEvent) -> Unit,
) {
    val colors = QuireTheme.colors
    var showThemeDialog by rememberSaveable { mutableStateOf(false) }
    var showFontDialog by rememberSaveable { mutableStateOf(false) }

    if (showThemeDialog) {
        ChooseThemeDialog(
            current = state.appTheme,
            onSelect = { theme ->
                onEvent(SettingsEvent.OnSetAppTheme(theme))
                showThemeDialog = false
            },
            onDismiss = { showThemeDialog = false },
        )
    }

    if (showFontDialog) {
        ChooseFontDialog(
            current = state.readerFont,
            onSelect = { font ->
                onEvent(SettingsEvent.OnSetReaderFont(font))
                showFontDialog = false
            },
            onDismiss = { showFontDialog = false },
        )
    }

    Column(modifier = Modifier.fillMaxWidth()) {
        WhatsAppSectionTitle(stringResource(Res.string.settings_display_section))

        val currentPalette = quireColors(state.appTheme)
        WhatsAppSettingsRow(
            icon = Res.drawable.ic_auto_fix_high,
            title = stringResource(Res.string.mobile_settings_app_theme),
            subtitle = state.appTheme.title,
            onClick = { showThemeDialog = true },
            trailing = {
                Row(
                    Modifier
                        .clip(CircleShape)
                        .background(currentPalette.background)
                        .border(1.dp, colors.paneDivider, CircleShape)
                        .padding(horizontal = 10.dp, vertical = 6.dp),
                    horizontalArrangement = Arrangement.spacedBy(5.dp),
                ) {
                    Box(Modifier.size(10.dp).clip(CircleShape).background(currentPalette.accent))
                    Box(Modifier.size(10.dp).clip(CircleShape).background(currentPalette.fillPrimary))
                }
            },
        )

        HorizontalDivider(
            color = colors.border.copy(alpha = 0.35f),
            modifier = Modifier.padding(vertical = 8.dp),
        )

        WhatsAppSectionTitle(stringResource(Res.string.mobile_settings_reading))

        // Live Reader Preview Card
        val readerSwatch = readerColors(state.readerTheme)
        val previewFontFamily = when (state.readerFont) {
            ReaderFont.Serif -> FontFamily.Serif
            ReaderFont.Sans -> FontFamily.SansSerif
            ReaderFont.Mono -> FontFamily.Monospace
        }
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 8.dp)
                .clip(RoundedCornerShape(20.dp))
                .background(readerSwatch.background)
                .border(1.dp, colors.border.copy(alpha = 0.25f), RoundedCornerShape(20.dp))
                .padding(18.dp),
        ) {
            Text(
                text = stringResource(Res.string.settings_reader_preview_sample),
                style = MaterialTheme.typography.bodyMedium.copy(
                    fontFamily = previewFontFamily,
                    fontSize = state.readerFontSize.sp,
                    lineHeight = (state.readerFontSize * 1.45f).sp,
                ),
                color = readerSwatch.text,
            )
        }

        // Page colours row
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 24.dp, vertical = 12.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = stringResource(Res.string.mobile_settings_reader_theme),
                    style = MaterialTheme.typography.bodyLarge.copy(fontSize = 16.sp),
                    color = colors.fillPrimary,
                )
                Text(
                    text = state.readerTheme.title,
                    style = MaterialTheme.typography.bodyMedium.copy(fontSize = 13.sp),
                    color = colors.fillSecondary,
                )
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                ReaderTheme.entries.forEach { theme ->
                    val selected = theme == state.readerTheme
                    val swatch = readerColors(theme)
                    Box(
                        Modifier
                            .size(44.dp)
                            .clip(CircleShape)
                            .background(swatch.background)
                            .border(
                                width = if (selected) 2.5.dp else 1.dp,
                                color = if (selected) colors.accent else colors.paneDivider,
                                shape = CircleShape,
                            )
                            .clickable(onClickLabel = theme.title) {
                                onEvent(SettingsEvent.OnSetReaderTheme(theme))
                            },
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = "Aa",
                            color = swatch.text,
                            style = MaterialTheme.typography.labelLarge.copy(
                                fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                            ),
                        )
                    }
                }
            }
        }

        // Typeface row (opens dialog)
        WhatsAppSettingsRow(
            icon = Res.drawable.ic_description,
            title = stringResource(Res.string.mobile_settings_reader_font),
            subtitle = state.readerFont.title,
            onClick = { showFontDialog = true },
        )

        // Text size slider row
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 24.dp, vertical = 10.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = stringResource(Res.string.mobile_settings_reader_size),
                    style = MaterialTheme.typography.bodyLarge.copy(fontSize = 16.sp),
                    color = colors.fillPrimary,
                )
                Text(
                    text = "${state.readerFontSize} pt",
                    style = MaterialTheme.typography.bodyMedium.copy(
                        fontSize = 13.sp,
                        fontWeight = FontWeight.SemiBold,
                    ),
                    color = colors.accent,
                )
            }
            Slider(
                value = state.readerFontSize.toFloat(),
                onValueChange = { onEvent(SettingsEvent.OnSetReaderFontSize(it.toInt())) },
                valueRange = 12f..28f,
                steps = 15,
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}

@Composable
private fun ChooseThemeDialog(
    current: AppTheme,
    onSelect: (AppTheme) -> Unit,
    onDismiss: () -> Unit,
) {
    val colors = QuireTheme.colors
    val systemHint = stringResource(Res.string.mobile_settings_system_hint)
    DialogShell(
        title = stringResource(Res.string.settings_choose_theme),
        icon = Res.drawable.ic_auto_fix_high,
        onDismiss = onDismiss,
        modifier = Modifier.padding(horizontal = Dimens.SpaceMd),
        actions = {
            DialogCancel(onClick = onDismiss)
        },
    ) {
        Column(modifier = Modifier.fillMaxWidth()) {
            AppTheme.entries.forEach { theme ->
                val palette = quireColors(theme)
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(Dimens.RadiusBase))
                        .clickable { onSelect(theme) }
                        .padding(horizontal = 4.dp, vertical = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    RadioButton(
                        selected = theme == current,
                        onClick = { onSelect(theme) },
                    )
                    Spacer(Modifier.width(8.dp))
                    Column(Modifier.weight(1f)) {
                        Text(
                            text = theme.title,
                            style = MaterialTheme.typography.bodyLarge,
                            color = colors.fillPrimary,
                        )
                        if (theme == AppTheme.System) {
                            Text(
                                text = systemHint,
                                style = MaterialTheme.typography.bodySmall,
                                color = colors.fillSecondary,
                            )
                        }
                    }
                    Row(
                        Modifier
                            .clip(CircleShape)
                            .background(palette.background)
                            .border(1.dp, colors.paneDivider, CircleShape)
                            .padding(horizontal = 8.dp, vertical = 5.dp),
                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        Box(Modifier.size(10.dp).clip(CircleShape).background(palette.accent))
                        Box(Modifier.size(10.dp).clip(CircleShape).background(palette.fillPrimary))
                    }
                }
            }
        }
    }
}

@Composable
private fun ChooseFontDialog(
    current: ReaderFont,
    onSelect: (ReaderFont) -> Unit,
    onDismiss: () -> Unit,
) {
    val colors = QuireTheme.colors
    DialogShell(
        title = stringResource(Res.string.settings_choose_font),
        icon = Res.drawable.ic_description,
        onDismiss = onDismiss,
        modifier = Modifier.padding(horizontal = Dimens.SpaceMd),
        actions = {
            DialogCancel(onClick = onDismiss)
        },
    ) {
        Column(modifier = Modifier.fillMaxWidth()) {
            ReaderFont.entries.forEach { font ->
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(Dimens.RadiusBase))
                        .clickable { onSelect(font) }
                        .padding(horizontal = 4.dp, vertical = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    RadioButton(
                        selected = font == current,
                        onClick = { onSelect(font) },
                    )
                    Spacer(Modifier.width(8.dp))
                    Text(
                        text = font.title,
                        style = MaterialTheme.typography.bodyLarge.copy(
                            fontFamily = when (font) {
                                ReaderFont.Serif -> FontFamily.Serif
                                ReaderFont.Sans -> FontFamily.SansSerif
                                ReaderFont.Mono -> FontFamily.Monospace
                            },
                        ),
                        color = colors.fillPrimary,
                    )
                }
            }
        }
    }
}
