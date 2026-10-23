package dev.neurokitchen.quire.presentation.screens.settings.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.settings.SettingsSection
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_info
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_link
import dev.neurokitchen.quire.resources.ic_settings
import dev.neurokitchen.quire.resources.ic_subject
import org.jetbrains.compose.resources.painterResource

/** The panes of the settings window: modern category navigation. */
@Composable
fun SettingsSidebar(
    selected: SettingsSection,
    onClick: (SettingsSection) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    Column(
        modifier = modifier
            .fillMaxHeight()
            .background(colors.sidepane)
            .padding(Dimens.SpaceSm),
        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceMin),
    ) {
        SettingsSection.entries.forEach { section ->
            val isSelected = section == selected
            val icon = when (section) {
                SettingsSection.Appearance -> Res.drawable.ic_subject
                SettingsSection.Reader -> Res.drawable.ic_description
                SettingsSection.General -> Res.drawable.ic_settings
                SettingsSection.Libraries -> Res.drawable.ic_library_books
                SettingsSection.Export -> Res.drawable.ic_link
                SettingsSection.Shortcuts -> Res.drawable.ic_info
                SettingsSection.ConfigFile -> Res.drawable.ic_folder
                SettingsSection.About -> Res.drawable.ic_info
            }

            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(44.dp)
                    .clip(RoundedCornerShape(Dimens.RadiusFull))
                    // The selected section: a tinted pill, its icon and text in the accent.
                    .background(if (isSelected) colors.accent.copy(alpha = 0.16f) else Color.Transparent)
                    .clickable { onClick(section) }
                    .padding(horizontal = Dimens.SpaceSm),
            ) {
                Icon(
                    painterResource(icon),
                    contentDescription = null,
                    tint = if (isSelected) colors.accent else colors.fillSecondary,
                    modifier = Modifier.size(20.dp),
                )
                Spacer(Modifier.width(Dimens.SpaceSm))
                Text(
                    section.displayTitle,
                    style = MaterialTheme.typography.bodyLarge.copy(
                        fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal,
                    ),
                    color = if (isSelected) colors.accent else colors.fillPrimary,
                )
            }
        }
    }
}

val SettingsSection.displayTitle: String
    get() = when (this) {
        SettingsSection.Appearance -> "Appearance"
        SettingsSection.Reader -> "Reader Defaults"
        SettingsSection.General -> "General & Sync"
        SettingsSection.Libraries -> "Libraries"
        SettingsSection.Export -> "CiteKey & Export"
        SettingsSection.Shortcuts -> "Shortcuts"
        SettingsSection.ConfigFile -> "Configuration (TOML)"
        SettingsSection.About -> "About & Legal"
    }
