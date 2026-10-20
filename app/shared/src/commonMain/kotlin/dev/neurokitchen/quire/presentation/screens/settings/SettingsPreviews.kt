package dev.neurokitchen.quire.presentation.screens.settings

import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview
import dev.neurokitchen.quire.domain.model.SettingKind
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme

@Preview(widthDp = 860, heightDp = 600)
@Composable
private fun SettingsScreenPreview() {
    val fields = listOf(
        SettingField("reader.width", "reader mode: text column width", SettingKind.Number, min = 40, max = 120, saved = "72"),
        SettingField("reader.justify", "reader mode: justified text", SettingKind.Bool, saved = null),
        SettingField("reader.theme", "reader mode: colors", SettingKind.Choice, listOf("dark", "light", "sepia"), saved = "sepia"),
    )
    QuireTheme(desktop = true) {
        Surface {
            SettingsScreen(
                SettingsState(version = "0.1.0", isLoading = false, configPath = "~/.config/quire/config.toml", section = SettingsSection.Reader, fields = fields),
                onEvent = {},
            )
        }
    }
}
