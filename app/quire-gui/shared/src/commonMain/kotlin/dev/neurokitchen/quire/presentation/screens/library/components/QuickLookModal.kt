package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mapping.kindIcon
import dev.neurokitchen.quire.presentation.screens.library.ReferenceDetailState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_content_copy
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import org.jetbrains.compose.resources.painterResource

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun QuickLookModal(
    reference: ReferenceDetailState,
    onDismissRequest: () -> Unit,
    onOpenReader: () -> Unit,
) {
    val colors = QuireTheme.colors
    val clipboard = LocalClipboardManager.current
    val scrollState = rememberScrollState()

    Dialog(onDismissRequest = onDismissRequest) {
        Surface(
            shape = RoundedCornerShape(16.dp),
            color = colors.sidepane,
            shadowElevation = 16.dp,
            modifier = Modifier
                .width(680.dp)
                .heightIn(max = 640.dp)
                .border(1.dp, colors.border.copy(alpha = 0.8f), RoundedCornerShape(16.dp))
                .onKeyEvent { event ->
                    if (event.type == KeyEventType.KeyDown) {
                        when (event.key) {
                            Key.Spacebar, Key.Escape -> {
                                onDismissRequest()
                                true
                            }
                            Key.Enter -> {
                                onOpenReader()
                                true
                            }
                            else -> false
                        }
                    } else false
                },
        ) {
            Column(Modifier.fillMaxWidth()) {
                // Top Header Bar
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 20.dp, vertical = 14.dp),
                ) {
                    // CSL Kind Badge
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier
                            .clip(RoundedCornerShape(Dimens.RadiusSmall))
                            .background(colors.accent.copy(alpha = 0.15f))
                            .padding(horizontal = 8.dp, vertical = 4.dp),
                    ) {
                        Icon(
                            painter = painterResource(kindIcon(reference.kind)),
                            contentDescription = null,
                            tint = colors.accent,
                            modifier = Modifier.size(14.dp),
                        )
                        Spacer(Modifier.width(6.dp))
                        Text(
                            reference.kind.replace("_", " ").uppercase(),
                            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold, fontSize = 10.sp),
                            color = colors.accent,
                        )
                    }

                    if (reference.year.isNotBlank()) {
                        Spacer(Modifier.width(8.dp))
                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                .background(colors.fillQuarternary)
                                .padding(horizontal = 8.dp, vertical = 4.dp),
                        ) {
                            Text(
                                reference.year,
                                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.SemiBold, fontSize = 10.sp),
                                color = colors.fillPrimary,
                            )
                        }
                    }

                    // Reading Status Dot & Label
                    Spacer(Modifier.width(8.dp))
                    val statusColor = when (reference.reading) {
                        ReadingStatus.Read -> colors.statusRead
                        ReadingStatus.Reading -> colors.statusReading
                        ReadingStatus.Unread -> colors.statusUnread
                    }
                    val statusText = when (reference.reading) {
                        ReadingStatus.Read -> "Read"
                        ReadingStatus.Reading -> "Reading"
                        ReadingStatus.Unread -> "Unread"
                    }
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier
                            .clip(RoundedCornerShape(Dimens.RadiusSmall))
                            .background(statusColor.copy(alpha = 0.15f))
                            .padding(horizontal = 8.dp, vertical = 4.dp),
                    ) {
                        Box(
                            modifier = Modifier
                                .size(6.dp)
                                .clip(CircleShape)
                                .background(statusColor),
                        )
                        Spacer(Modifier.width(6.dp))
                        Text(
                            statusText,
                            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Medium, fontSize = 10.sp),
                            color = statusColor,
                        )
                    }

                    if (reference.hasPdf) {
                        Spacer(Modifier.width(8.dp))
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                .background(colors.accentRed.copy(alpha = 0.15f))
                                .padding(horizontal = 8.dp, vertical = 4.dp),
                        ) {
                            Icon(
                                painterResource(Res.drawable.ic_picture_as_pdf),
                                contentDescription = null,
                                tint = colors.accentRed,
                                modifier = Modifier.size(13.dp),
                            )
                            Spacer(Modifier.width(4.dp))
                            Text(
                                "PDF Available",
                                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.SemiBold, fontSize = 10.sp),
                                color = colors.accentRed,
                            )
                        }
                    }

                    Spacer(Modifier.weight(1f))

                    // Close Button
                    Box(
                        contentAlignment = Alignment.Center,
                        modifier = Modifier
                            .size(28.dp)
                            .clip(CircleShape)
                            .background(colors.fillQuarternary.copy(alpha = 0.6f))
                            .clickable(onClick = onDismissRequest),
                    ) {
                        Icon(
                            painterResource(Res.drawable.ic_close),
                            contentDescription = "Close Preview",
                            tint = colors.fillSecondary,
                            modifier = Modifier.size(16.dp),
                        )
                    }
                }

                HorizontalDivider(color = colors.border.copy(alpha = 0.4f))

                // Scrollable Body Content
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .weight(1f, fill = false)
                        .verticalScroll(scrollState)
                        .padding(horizontal = 24.dp, vertical = 16.dp),
                ) {
                    // Paper Title
                    Text(
                        text = reference.title,
                        style = MaterialTheme.typography.headlineSmall.copy(
                            fontWeight = FontWeight.Bold,
                            fontSize = 18.sp,
                            lineHeight = 24.sp,
                        ),
                        color = colors.fillPrimary,
                    )

                    Spacer(Modifier.height(10.dp))

                    // CiteKey and Copy Action
                    if (reference.citeKey.isNotBlank()) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                .background(colors.fillQuarternary.copy(alpha = 0.6f))
                                .clickable { clipboard.setText(AnnotatedString("@${reference.citeKey}")) }
                                .padding(horizontal = 8.dp, vertical = 4.dp),
                        ) {
                            Text(
                                "@${reference.citeKey}",
                                style = MaterialTheme.typography.labelSmall.copy(
                                    fontWeight = FontWeight.SemiBold,
                                    fontSize = 11.sp,
                                ),
                                color = colors.accent,
                            )
                            Spacer(Modifier.width(6.dp))
                            Icon(
                                painterResource(Res.drawable.ic_content_copy),
                                contentDescription = "Copy CiteKey",
                                tint = colors.fillSecondary,
                                modifier = Modifier.size(12.dp),
                            )
                        }
                        Spacer(Modifier.height(10.dp))
                    }

                    // Authors
                    val authorsText = if (reference.authors.isNotEmpty()) {
                        reference.authors.joinToString(" • ")
                    } else if (reference.creator.isNotBlank()) {
                        reference.creator
                    } else {
                        "Unknown Author"
                    }
                    Text(
                        text = authorsText,
                        style = MaterialTheme.typography.bodyMedium.copy(
                            fontWeight = FontWeight.Medium,
                            fontSize = 13.sp,
                        ),
                        color = colors.fillSecondary,
                    )

                    // Abstract Section
                    if (reference.abstract.isNotBlank()) {
                        Spacer(Modifier.height(16.dp))
                        Text(
                            "ABSTRACT",
                            style = MaterialTheme.typography.labelSmall.copy(
                                fontWeight = FontWeight.Bold,
                                fontSize = 10.sp,
                                letterSpacing = 1.sp,
                            ),
                            color = colors.fillTertiary,
                        )
                        Spacer(Modifier.height(6.dp))
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(Dimens.RadiusBase))
                                .background(colors.background.copy(alpha = 0.6f))
                                .border(1.dp, colors.border.copy(alpha = 0.4f), RoundedCornerShape(Dimens.RadiusBase))
                                .padding(14.dp),
                        ) {
                            Text(
                                text = reference.abstract,
                                style = MaterialTheme.typography.bodyMedium.copy(
                                    fontSize = 12.sp,
                                    lineHeight = 18.sp,
                                ),
                                color = colors.fillPrimary.copy(alpha = 0.88f),
                            )
                        }
                    }

                    // Tags & Collections
                    if (reference.tags.isNotEmpty() || reference.projects.isNotEmpty()) {
                        Spacer(Modifier.height(16.dp))
                        FlowRow(
                            horizontalArrangement = Arrangement.spacedBy(6.dp),
                            verticalArrangement = Arrangement.spacedBy(6.dp),
                        ) {
                            reference.projects.forEach { project ->
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                        .background(colors.fillQuarternary.copy(alpha = 0.8f))
                                        .padding(horizontal = 8.dp, vertical = 3.dp),
                                ) {
                                    Icon(
                                        painterResource(Res.drawable.ic_folder),
                                        contentDescription = null,
                                        tint = colors.fillSecondary,
                                        modifier = Modifier.size(12.dp),
                                    )
                                    Spacer(Modifier.width(4.dp))
                                    Text(
                                        project,
                                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp),
                                        color = colors.fillPrimary,
                                    )
                                }
                            }
                            reference.tags.forEach { tag ->
                                Box(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(Dimens.RadiusFull))
                                        .background(colors.accent.copy(alpha = 0.12f))
                                        .padding(horizontal = 9.dp, vertical = 3.dp),
                                ) {
                                    Text(
                                        "#$tag",
                                        style = MaterialTheme.typography.labelSmall.copy(
                                            fontSize = 11.sp,
                                            fontWeight = FontWeight.Medium,
                                        ),
                                        color = colors.accent,
                                    )
                                }
                            }
                        }
                    }
                }

                HorizontalDivider(color = colors.border.copy(alpha = 0.4f))

                // Footer Bottom Action Bar
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(colors.tabbar.copy(alpha = 0.5f))
                        .padding(horizontal = 20.dp, vertical = 12.dp),
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        Text(
                            "Press Space or Esc to dismiss",
                            style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp),
                            color = colors.fillTertiary,
                        )
                    }

                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        OutlinedButton(
                            onClick = onDismissRequest,
                            shape = RoundedCornerShape(Dimens.RadiusSmall),
                        ) {
                            Text("Close", style = MaterialTheme.typography.labelMedium)
                        }

                        Button(
                            onClick = onOpenReader,
                            shape = RoundedCornerShape(Dimens.RadiusSmall),
                            colors = ButtonDefaults.buttonColors(
                                containerColor = colors.accent,
                                contentColor = colors.onAccent,
                            ),
                        ) {
                            Text("Open in Reader", style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.Bold))
                        }
                    }
                }
            }
        }
    }
}
