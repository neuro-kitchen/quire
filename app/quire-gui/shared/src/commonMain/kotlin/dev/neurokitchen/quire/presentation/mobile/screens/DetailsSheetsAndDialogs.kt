package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogConfirm
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsGroup
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsItem
import dev.neurokitchen.quire.presentation.screens.library.ReferenceDetailState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.common_link_paper
import dev.neurokitchen.quire.resources.details_add
import dev.neurokitchen.quire.resources.details_add_note
import dev.neurokitchen.quire.resources.details_add_tag
import dev.neurokitchen.quire.resources.details_attach_from_device
import dev.neurokitchen.quire.resources.details_attach_from_device_sub
import dev.neurokitchen.quire.resources.details_attachments_subtitle
import dev.neurokitchen.quire.resources.details_delete_message
import dev.neurokitchen.quire.resources.details_delete_title
import dev.neurokitchen.quire.resources.details_find_pdf_sub
import dev.neurokitchen.quire.resources.details_note_new
import dev.neurokitchen.quire.resources.details_note_sheet_subtitle
import dev.neurokitchen.quire.resources.details_projects_none
import dev.neurokitchen.quire.resources.details_projects_sheet_subtitle
import dev.neurokitchen.quire.resources.details_projects_sheet_title
import dev.neurokitchen.quire.resources.details_read
import dev.neurokitchen.quire.resources.details_remove_pdfs_sub
import dev.neurokitchen.quire.resources.details_tags_add_subtitle
import dev.neurokitchen.quire.resources.details_tags_add_title
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_create_new_folder
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_link
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import dev.neurokitchen.quire.resources.ic_sell
import dev.neurokitchen.quire.resources.ic_upload
import dev.neurokitchen.quire.resources.item_delete
import dev.neurokitchen.quire.resources.item_get_pdf
import dev.neurokitchen.quire.resources.item_remove_pdfs
import dev.neurokitchen.quire.resources.library_new_collection
import dev.neurokitchen.quire.resources.name_create
import dev.neurokitchen.quire.resources.section_attachments
import dev.neurokitchen.quire.resources.section_empty_attachments
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun ProjectsBottomSheet(
    reference: ReferenceDetailState,
    projectChoices: List<Pair<String, String>>,
    onToggleProject: (String, Boolean) -> Unit,
    onNewProject: () -> Unit,
    onDismiss: () -> Unit,
) {
    val colors = QuireTheme.colors
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = colors.cardBackground,
        contentColor = colors.fillPrimary,
        tonalElevation = 8.dp,
        shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(bottom = 24.dp),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 20.dp, vertical = 8.dp),
            ) {
                Column(Modifier.weight(1f)) {
                    Text(
                        text = stringResource(Res.string.details_projects_sheet_title),
                        style = MaterialTheme.typography.titleMedium.copy(
                            fontWeight = FontWeight.Bold,
                            fontSize = 18.sp,
                        ),
                        color = colors.fillPrimary,
                    )
                    Text(
                        text = stringResource(Res.string.details_projects_sheet_subtitle),
                        style = MaterialTheme.typography.bodySmall.copy(fontSize = 12.sp),
                        color = colors.fillSecondary,
                    )
                }

                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Box(
                        modifier = Modifier
                            .size(36.dp)
                            .clip(CircleShape)
                            .background(colors.fillQuinary)
                            .clickable(onClick = onNewProject),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            painter = painterResource(Res.drawable.ic_create_new_folder),
                            contentDescription = stringResource(Res.string.library_new_collection),
                            tint = colors.fillPrimary,
                            modifier = Modifier.size(18.dp),
                        )
                    }
                    Box(
                        modifier = Modifier
                            .size(36.dp)
                            .clip(CircleShape)
                            .background(colors.fillQuinary)
                            .clickable(onClick = onDismiss),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            painter = painterResource(Res.drawable.ic_close),
                            contentDescription = null,
                            tint = colors.fillSecondary,
                            modifier = Modifier.size(18.dp),
                        )
                    }
                }
            }

            HorizontalDivider(
                color = colors.border.copy(alpha = 0.25f),
                modifier = Modifier.padding(vertical = 8.dp),
            )

            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 12.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                if (projectChoices.isEmpty()) {
                    Text(
                        text = stringResource(Res.string.details_projects_none),
                        style = MaterialTheme.typography.bodyMedium,
                        color = colors.fillSecondary,
                        modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp),
                    )
                } else {
                    projectChoices.forEach { (path, name) ->
                        val isAssigned = path in reference.projectPaths
                        val depth = path.count { it == '/' }
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(Dimens.RadiusMedium))
                                .background(if (isAssigned) colors.accent.copy(alpha = 0.14f) else Color.Transparent)
                                .clickable { onToggleProject(path, isAssigned) }
                                .padding(start = (depth * 18 + 12).dp, end = 12.dp, top = 12.dp, bottom = 12.dp),
                        ) {
                            Icon(
                                painter = painterResource(Res.drawable.ic_folder),
                                contentDescription = null,
                                tint = if (isAssigned) colors.accent else colors.fillSecondary,
                                modifier = Modifier.size(20.dp),
                            )
                            Spacer(Modifier.width(12.dp))
                            Text(
                                text = name,
                                style = MaterialTheme.typography.bodyMedium.copy(
                                    fontWeight = if (isAssigned) FontWeight.Bold else FontWeight.Medium,
                                    fontSize = 15.sp,
                                ),
                                color = if (isAssigned) colors.accent else colors.fillPrimary,
                                modifier = Modifier.weight(1f),
                            )
                            if (isAssigned) {
                                Box(
                                    modifier = Modifier
                                        .size(8.dp)
                                        .clip(CircleShape)
                                        .background(colors.accent),
                                )
                            }
                        }
                    }
                }

                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(Dimens.RadiusMedium))
                        .clickable(onClick = onNewProject)
                        .padding(horizontal = 12.dp, vertical = 12.dp),
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_create_new_folder),
                        contentDescription = null,
                        tint = colors.accent,
                        modifier = Modifier.size(20.dp),
                    )
                    Spacer(Modifier.width(12.dp))
                    Text(
                        text = stringResource(Res.string.library_new_collection),
                        style = MaterialTheme.typography.bodyMedium.copy(
                            fontWeight = FontWeight.SemiBold,
                            fontSize = 15.sp,
                        ),
                        color = colors.accent,
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun AttachmentsBottomSheet(
    reference: ReferenceDetailState,
    onOpenPdf: () -> Unit,
    onAttachFromDevice: () -> Unit,
    onFindPdf: () -> Unit,
    onRemovePdfs: () -> Unit,
    onDismiss: () -> Unit,
) {
    val colors = QuireTheme.colors
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = colors.cardBackground,
        contentColor = colors.fillPrimary,
        tonalElevation = 8.dp,
        shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 20.dp, vertical = 4.dp),
            ) {
                Column(Modifier.weight(1f)) {
                    Text(
                        text = stringResource(Res.string.section_attachments),
                        style = MaterialTheme.typography.titleMedium.copy(
                            fontWeight = FontWeight.Bold,
                            fontSize = 18.sp,
                        ),
                        color = colors.fillPrimary,
                    )
                    Text(
                        text = stringResource(Res.string.details_attachments_subtitle),
                        style = MaterialTheme.typography.bodySmall.copy(fontSize = 12.sp),
                        color = colors.fillSecondary,
                    )
                }
                Box(
                    modifier = Modifier
                        .size(36.dp)
                        .clip(CircleShape)
                        .background(colors.fillQuinary)
                        .clickable(onClick = onDismiss),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_close),
                        contentDescription = null,
                        tint = colors.fillSecondary,
                        modifier = Modifier.size(18.dp),
                    )
                }
            }

            if (reference.pdfFiles.isEmpty()) {
                Text(
                    text = stringResource(Res.string.section_empty_attachments),
                    style = MaterialTheme.typography.bodyMedium,
                    color = colors.fillSecondary,
                    modifier = Modifier.padding(horizontal = 20.dp, vertical = 4.dp),
                )
            } else {
                val fileRows = buildList<@Composable (Shape) -> Unit> {
                    reference.pdfFiles.forEach { file ->
                        val fileName = file.substringAfterLast('/').substringAfterLast('\\')
                        add { shape ->
                            SettingsItem(
                                title = fileName,
                                supporting = stringResource(Res.string.details_read),
                                icon = Res.drawable.ic_picture_as_pdf,
                                shape = shape,
                                opensPage = true,
                                onClick = onOpenPdf,
                            )
                        }
                    }
                }
                SettingsGroup(
                    rows = fileRows,
                    modifier = Modifier.padding(horizontal = Dimens.SpaceMd),
                )
            }

            val actionRows = buildList<@Composable (Shape) -> Unit> {
                add { shape ->
                    SettingsItem(
                        title = stringResource(Res.string.details_attach_from_device),
                        supporting = stringResource(Res.string.details_attach_from_device_sub),
                        icon = Res.drawable.ic_upload,
                        shape = shape,
                        onClick = onAttachFromDevice,
                    )
                }
                if (reference.pdfFiles.isEmpty()) {
                    add { shape ->
                        SettingsItem(
                            title = stringResource(Res.string.item_get_pdf),
                            supporting = stringResource(Res.string.details_find_pdf_sub),
                            icon = Res.drawable.ic_picture_as_pdf,
                            shape = shape,
                            onClick = onFindPdf,
                        )
                    }
                } else {
                    add { shape ->
                        SettingsItem(
                            title = stringResource(Res.string.item_remove_pdfs),
                            supporting = stringResource(Res.string.details_remove_pdfs_sub),
                            icon = Res.drawable.ic_delete,
                            shape = shape,
                            onClick = onRemovePdfs,
                        )
                    }
                }
            }
            SettingsGroup(
                rows = actionRows,
                modifier = Modifier.padding(horizontal = Dimens.SpaceMd),
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun NoteComposerSheet(
    draft: String,
    onDraftChange: (String) -> Unit,
    onSave: () -> Unit,
    onDismiss: () -> Unit,
) {
    val colors = QuireTheme.colors
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = colors.cardBackground,
        contentColor = colors.fillPrimary,
        tonalElevation = 8.dp,
        shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(horizontal = 20.dp)
                .padding(bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Column(Modifier.weight(1f)) {
                    Text(
                        text = stringResource(Res.string.details_note_new),
                        style = MaterialTheme.typography.titleMedium.copy(
                            fontWeight = FontWeight.Bold,
                            fontSize = 18.sp,
                        ),
                        color = colors.fillPrimary,
                    )
                    Text(
                        text = stringResource(Res.string.details_note_sheet_subtitle),
                        style = MaterialTheme.typography.bodySmall.copy(fontSize = 12.sp),
                        color = colors.fillSecondary,
                    )
                }
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(Dimens.RadiusFull))
                        .background(colors.accent.copy(alpha = 0.14f))
                        .clickable {
                            val next = if (draft.isEmpty() || draft.endsWith(" ")) "$draft[[key]]" else "$draft [[key]]"
                            onDraftChange(next)
                        }
                        .padding(horizontal = 10.dp, vertical = 6.dp),
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        Icon(
                            painter = painterResource(Res.drawable.ic_link),
                            contentDescription = null,
                            tint = colors.accent,
                            modifier = Modifier.size(14.dp),
                        )
                        Text(
                            text = stringResource(Res.string.common_link_paper),
                            style = MaterialTheme.typography.labelSmall,
                            fontWeight = FontWeight.SemiBold,
                            color = colors.accent,
                        )
                    }
                }
            }

            OutlinedTextField(
                value = draft,
                onValueChange = onDraftChange,
                placeholder = { Text(stringResource(Res.string.details_add_note)) },
                minLines = 3,
                maxLines = 8,
                shape = RoundedCornerShape(Dimens.RadiusBase),
                modifier = Modifier.fillMaxWidth(),
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs, Alignment.End),
            ) {
                DialogCancel(onClick = onDismiss)
                DialogConfirm(
                    label = stringResource(Res.string.details_add),
                    onClick = onSave,
                    enabled = draft.isNotBlank(),
                )
            }
        }
    }
}

@Composable
internal fun AddTagDialog(
    draft: String,
    onDraftChange: (String) -> Unit,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    DialogShell(
        title = stringResource(Res.string.details_tags_add_title),
        subtitle = stringResource(Res.string.details_tags_add_subtitle),
        icon = Res.drawable.ic_sell,
        onDismiss = onDismiss,
        modifier = Modifier.padding(horizontal = Dimens.SpaceMd),
        actions = {
            DialogCancel(onClick = onDismiss)
            DialogConfirm(
                label = stringResource(Res.string.details_add),
                onClick = onConfirm,
                enabled = draft.isNotBlank(),
            )
        },
    ) {
        OutlinedTextField(
            value = draft,
            onValueChange = onDraftChange,
            label = { Text(stringResource(Res.string.details_add_tag)) },
            singleLine = true,
            shape = RoundedCornerShape(Dimens.RadiusBase),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { if (draft.isNotBlank()) onConfirm() }),
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

@Composable
internal fun NewArticleProjectDialog(
    onDismiss: () -> Unit,
    onCreate: (String) -> Unit,
) {
    var name by rememberSaveable { mutableStateOf("") }
    DialogShell(
        title = stringResource(Res.string.library_new_collection),
        icon = Res.drawable.ic_create_new_folder,
        onDismiss = onDismiss,
        modifier = Modifier.padding(horizontal = Dimens.SpaceMd),
        actions = {
            DialogCancel(onClick = onDismiss)
            DialogConfirm(
                label = stringResource(Res.string.name_create),
                onClick = { onCreate(name.trim()) },
                enabled = name.isNotBlank(),
            )
        },
    ) {
        OutlinedTextField(
            value = name,
            onValueChange = { name = it },
            label = { Text(stringResource(Res.string.library_new_collection)) },
            singleLine = true,
            shape = RoundedCornerShape(Dimens.RadiusBase),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { if (name.isNotBlank()) onCreate(name.trim()) }),
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

@Composable
internal fun DeleteArticleDialog(
    title: String,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    DialogShell(
        title = stringResource(Res.string.details_delete_title),
        subtitle = stringResource(Res.string.details_delete_message, title),
        icon = Res.drawable.ic_delete,
        iconTint = MaterialTheme.colorScheme.error,
        onDismiss = onDismiss,
        modifier = Modifier.padding(horizontal = Dimens.SpaceMd),
        actions = {
            DialogCancel(onClick = onDismiss)
            DialogConfirm(
                label = stringResource(Res.string.item_delete),
                onClick = onConfirm,
                destructive = true,
            )
        },
    ) {}
}
