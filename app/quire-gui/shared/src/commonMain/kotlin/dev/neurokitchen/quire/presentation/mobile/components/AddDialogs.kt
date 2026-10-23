package dev.neurokitchen.quire.presentation.mobile.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_link
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import org.jetbrains.compose.resources.painterResource

/**
 * Dialog for adding a paper by DOI or web link.
 */
@Composable
fun AddDoiOrLinkDialog(
    onDismiss: () -> Unit,
    onConfirm: (String) -> Unit,
) {
    var text by remember { mutableStateOf("") }
    val colors = QuireTheme.colors

    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(
                    painter = painterResource(Res.drawable.ic_link),
                    contentDescription = null,
                    tint = colors.accent,
                    modifier = Modifier.size(24.dp),
                )
                Spacer(Modifier.width(10.dp))
                Text(
                    text = "Add by DOI or Link",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold,
                    color = colors.fillPrimary,
                )
            }
        },
        text = {
            Column {
                Text(
                    text = "Enter or paste a DOI (e.g. 10.1038/s41586-020-2649-2) or a paper URL.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = colors.fillSecondary,
                )
                Spacer(Modifier.height(14.dp))
                OutlinedTextField(
                    value = text,
                    onValueChange = { text = it },
                    placeholder = {
                        Text("10.1038/... or https://...", style = MaterialTheme.typography.bodyMedium)
                    },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                )
            }
        },
        confirmButton = {
            Button(
                onClick = {
                    if (text.isNotBlank()) {
                        onConfirm(text.trim())
                    }
                },
                enabled = text.isNotBlank(),
                colors = ButtonDefaults.buttonColors(
                    containerColor = colors.accent,
                    contentColor = colors.onAccent,
                ),
                shape = RoundedCornerShape(12.dp),
            ) {
                Text("Add")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Cancel", color = colors.fillSecondary)
            }
        },
        containerColor = colors.cardBackground,
        shape = RoundedCornerShape(20.dp),
    )
}

/**
 * Prompt menu for opening / picking files (.pdf or .bib).
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddFilePromptSheet(
    onDismiss: () -> Unit,
    onPickPdf: () -> Unit,
    onPickBib: () -> Unit,
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val colors = QuireTheme.colors

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = colors.cardBackground,
        contentColor = colors.fillPrimary,
        shape = RoundedCornerShape(topStart = 20.dp, topEnd = 20.dp),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(bottom = Dimens.SpaceMd),
        ) {
            // Header
            Text(
                text = "Open File",
                style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
                color = colors.fillPrimary,
                modifier = Modifier.padding(horizontal = Dimens.SpaceLg),
            )
            Text(
                text = "Choose the document format to add to your library",
                style = MaterialTheme.typography.bodySmall,
                color = colors.fillSecondary,
                modifier = Modifier.padding(horizontal = Dimens.SpaceLg, vertical = 2.dp),
            )

            Spacer(Modifier.height(Dimens.SpaceSm))
            HorizontalDivider(color = colors.paneDivider)
            Spacer(Modifier.height(Dimens.SpaceXs))

            // Option 1: PDF Document
            FileOptionRow(
                icon = Res.drawable.ic_picture_as_pdf,
                iconTint = colors.accent,
                title = "PDF Document (.pdf)",
                subtitle = "Add an individual paper with PDF attached",
                onClick = {
                    onDismiss()
                    onPickPdf()
                },
            )

            // Option 2: BibTeX File
            FileOptionRow(
                icon = Res.drawable.ic_description,
                iconTint = colors.fillPrimary,
                title = "BibTeX File (.bib)",
                subtitle = "Import reference entries from a .bib collection",
                onClick = {
                    onDismiss()
                    onPickBib()
                },
            )
        }
    }
}

@Composable
private fun FileOptionRow(
    icon: org.jetbrains.compose.resources.DrawableResource,
    iconTint: Color,
    title: String,
    subtitle: String,
    onClick: () -> Unit,
) {
    val colors = QuireTheme.colors
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = Dimens.SpaceLg, vertical = 14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .size(40.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(iconTint.copy(alpha = 0.12f)),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                painter = painterResource(icon),
                contentDescription = null,
                tint = iconTint,
                modifier = Modifier.size(22.dp),
            )
        }
        Spacer(Modifier.width(14.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge.copy(
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 15.sp,
                ),
                color = colors.fillPrimary,
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall.copy(fontSize = 12.sp),
                color = colors.fillSecondary,
            )
        }
    }
}
