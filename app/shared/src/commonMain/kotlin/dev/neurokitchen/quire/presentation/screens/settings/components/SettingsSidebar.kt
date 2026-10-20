package dev.neurokitchen.quire.presentation.screens.settings.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.settings.SettingsSection
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.settings_section_export
import dev.neurokitchen.quire.resources.settings_section_file
import dev.neurokitchen.quire.resources.settings_section_general
import dev.neurokitchen.quire.resources.settings_section_libraries
import dev.neurokitchen.quire.resources.settings_section_reader
import dev.neurokitchen.quire.resources.settings_section_sync
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

/** The panes of the settings window, like Zotero's settings sidebar. */
@Composable
fun SettingsSidebar(
    selected: SettingsSection,
    onClick: (SettingsSection) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    Column(modifier.fillMaxHeight().background(colors.sidepane).padding(Dimens.SpaceXs)) {
        SettingsSection.entries.forEach { section ->
            val isSelected = section == selected
            Text(
                stringResource(section.label),
                style = MaterialTheme.typography.bodyMedium,
                color = if (isSelected) colors.onAccent else colors.fillPrimary,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(Dimens.RowHeight)
                    .clip(RoundedCornerShape(Dimens.RadiusLarge))
                    .background(if (isSelected) colors.accent else Color.Transparent)
                    .clickable { onClick(section) }
                    .padding(horizontal = Dimens.SpaceXs, vertical = 4.dp),
            )
        }
    }
}

val SettingsSection.label: StringResource
    get() = when (this) {
        SettingsSection.General -> Res.string.settings_section_general
        SettingsSection.Libraries -> Res.string.settings_section_libraries
        SettingsSection.Sync -> Res.string.settings_section_sync
        SettingsSection.Export -> Res.string.settings_section_export
        SettingsSection.Reader -> Res.string.settings_section_reader
        SettingsSection.ConfigFile -> Res.string.settings_section_file
    }
