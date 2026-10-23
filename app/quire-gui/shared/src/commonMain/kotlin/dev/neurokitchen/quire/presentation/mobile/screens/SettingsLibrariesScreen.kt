package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
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
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.presentation.components.notice.NoticeBar
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogConfirm
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsGroup
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsItem
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.screens.tools.ToolsEvent
import dev.neurokitchen.quire.presentation.screens.tools.ToolsSection
import dev.neurokitchen.quire.presentation.screens.tools.ToolsViewModel
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_create_new_folder
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.library_my_library
import dev.neurokitchen.quire.resources.mobile_settings_libraries
import dev.neurokitchen.quire.resources.mobile_settings_libraries_empty
import dev.neurokitchen.quire.resources.mobile_settings_library_default_sub
import dev.neurokitchen.quire.resources.mobile_settings_library_switch_sub
import dev.neurokitchen.quire.resources.mobile_settings_new_library_slug_hint
import dev.neurokitchen.quire.resources.mobile_settings_new_library_title
import dev.neurokitchen.quire.resources.tools_create
import dev.neurokitchen.quire.resources.tools_default
import dev.neurokitchen.quire.resources.tools_libraries_hint
import dev.neurokitchen.quire.resources.tools_make_default
import dev.neurokitchen.quire.resources.tools_new_library
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

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
