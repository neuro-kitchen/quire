package dev.neurokitchen.quire.presentation.screens.settings

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.settings.components.RawConfigEditor
import dev.neurokitchen.quire.presentation.screens.settings.components.SettingRow
import dev.neurokitchen.quire.presentation.screens.settings.components.SettingsSidebar
import dev.neurokitchen.quire.presentation.screens.settings.components.label
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.settings_file_location
import dev.neurokitchen.quire.resources.settings_load_failed
import dev.neurokitchen.quire.resources.settings_none_named
import dev.neurokitchen.quire.resources.settings_version
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

@Composable
fun SettingsRoute(viewModel: SettingsViewModel = koinViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    SettingsScreen(state, viewModel::onEvent)
}

@Composable
fun SettingsScreen(
    state: SettingsState,
    onEvent: (SettingsEvent) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    Row(modifier.fillMaxSize()) {
        SettingsSidebar(state.section, onClick = { onEvent(SettingsEvent.OnSectionClick(it)) }, Modifier.width(180.dp))
        VerticalDivider(color = colors.paneDivider)
        Column(Modifier.weight(1f).padding(Dimens.SpaceMd)) {
            Text(stringResource(state.section.label), style = MaterialTheme.typography.titleLarge, color = colors.fillPrimary)
            Text(
                stringResource(Res.string.settings_file_location, state.configPath),
                style = MaterialTheme.typography.bodySmall,
                color = colors.fillSecondary,
            )
            HorizontalDivider(color = colors.fillQuarternary, modifier = Modifier.padding(vertical = Dimens.SpaceXs))
            val error = state.error
            when {
                error != null -> Column {
                    Text(stringResource(Res.string.settings_load_failed), color = colors.accentRed, style = MaterialTheme.typography.titleMedium)
                    Text(error, color = colors.fillSecondary, style = MaterialTheme.typography.bodyMedium)
                }
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
}
