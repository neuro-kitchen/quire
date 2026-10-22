package dev.neurokitchen.quire.presentation.components.reference_card

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
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_format_quote
import dev.neurokitchen.quire.resources.ic_link
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import org.jetbrains.compose.resources.painterResource

/**
 * Mobile Card Long-Press Context Actions Sheet:
 * - Copy BibTeX
 * - Copy DOI
 * - Download PDF (if not already present)
 * - Delete Reference (styled in red with confirmation)
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReferenceActionsSheet(
    reference: ReferenceCardState,
    onDismiss: () -> Unit,
    onCopyBib: (ReferenceCardState) -> Unit,
    onCopyDoi: (ReferenceCardState) -> Unit,
    onDownloadPdf: ((ReferenceCardState) -> Unit)? = null,
    onDeleteRequest: (ReferenceCardState) -> Unit,
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
            // Header: Icon + Title + Creator/Year
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = Dimens.SpaceLg, vertical = Dimens.SpaceXs),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(
                    modifier = Modifier
                        .size(38.dp)
                        .clip(RoundedCornerShape(10.dp))
                        .background(
                            if (reference.hasPdf) colors.accent.copy(alpha = 0.15f)
                            else colors.fillQuinary,
                        ),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        painter = painterResource(
                            if (reference.hasPdf) Res.drawable.ic_picture_as_pdf
                            else Res.drawable.ic_description,
                        ),
                        contentDescription = null,
                        tint = if (reference.hasPdf) colors.accent else colors.fillSecondary,
                        modifier = Modifier.size(18.dp),
                    )
                }

                Spacer(Modifier.width(12.dp))

                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = reference.title,
                        style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Bold),
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                        color = colors.fillPrimary,
                    )
                    val sub = listOf(reference.creator, reference.year).filter { it.isNotBlank() }.joinToString(" · ")
                    if (sub.isNotBlank()) {
                        Text(
                            text = sub,
                            style = MaterialTheme.typography.bodySmall,
                            color = colors.fillSecondary,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }
            }

            Spacer(Modifier.height(Dimens.SpaceSm))
            HorizontalDivider(color = colors.paneDivider)
            Spacer(Modifier.height(Dimens.SpaceXs))

            // Action: Copy BibTeX
            ActionRow(
                icon = Res.drawable.ic_format_quote,
                title = "Copy BibTeX",
                subtitle = reference.citeKey.ifBlank { null },
                onClick = {
                    onCopyBib(reference)
                    onDismiss()
                },
            )

            // Action: Copy DOI
            ActionRow(
                icon = Res.drawable.ic_link,
                title = "Copy DOI",
                subtitle = reference.doi.ifBlank { "No DOI recorded" },
                enabled = reference.doi.isNotBlank(),
                onClick = {
                    onCopyDoi(reference)
                    onDismiss()
                },
            )

            // Action: Download PDF (only if not already present)
            if (!reference.hasPdf) {
                ActionRow(
                    icon = Res.drawable.ic_picture_as_pdf,
                    title = "Download PDF",
                    subtitle = "Fetch document attachment",
                    tint = colors.accent,
                    onClick = {
                        onDownloadPdf?.invoke(reference)
                        onDismiss()
                    },
                )
            }

            // Action: Delete Reference (styled in RED!)
            ActionRow(
                icon = Res.drawable.ic_delete,
                title = "Delete Reference",
                subtitle = "Move to Trash",
                tint = colors.accentRed,
                titleColor = colors.accentRed,
                onClick = {
                    onDeleteRequest(reference)
                },
            )
        }
    }
}

/**
 * Confirmation dialog shown before moving a reference to Trash.
 */
@Composable
fun DeleteReferenceConfirmationDialog(
    reference: ReferenceCardState,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    val colors = QuireTheme.colors
    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Text(
                text = "Delete Reference?",
                style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
                color = colors.fillPrimary,
            )
        },
        text = {
            Text(
                text = "Are you sure you want to move “${reference.title}” to Trash?",
                style = MaterialTheme.typography.bodyMedium,
                color = colors.fillSecondary,
            )
        },
        confirmButton = {
            TextButton(
                onClick = onConfirm,
                colors = ButtonDefaults.textButtonColors(contentColor = colors.accentRed),
            ) {
                Text(
                    text = "Delete",
                    fontWeight = FontWeight.Bold,
                    color = colors.accentRed,
                )
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(
                    text = "Cancel",
                    color = colors.fillSecondary,
                )
            }
        },
        containerColor = colors.cardBackground,
        shape = RoundedCornerShape(20.dp),
    )
}

@Composable
private fun ActionRow(
    icon: org.jetbrains.compose.resources.DrawableResource,
    title: String,
    subtitle: String? = null,
    tint: Color = QuireTheme.colors.fillPrimary,
    titleColor: Color = QuireTheme.colors.fillPrimary,
    enabled: Boolean = true,
    onClick: () -> Unit,
) {
    val colors = QuireTheme.colors
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = Dimens.SpaceLg, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            painter = painterResource(icon),
            contentDescription = null,
            tint = if (enabled) tint else colors.fillTertiary,
            modifier = Modifier.size(22.dp),
        )
        Spacer(Modifier.width(16.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge.copy(
                    fontWeight = FontWeight.Medium,
                    fontSize = 15.sp,
                ),
                color = if (enabled) titleColor else colors.fillTertiary,
            )
            if (subtitle != null) {
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.bodySmall.copy(fontSize = 12.sp),
                    color = colors.fillSecondary.copy(alpha = if (enabled) 0.8f else 0.5f),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}
