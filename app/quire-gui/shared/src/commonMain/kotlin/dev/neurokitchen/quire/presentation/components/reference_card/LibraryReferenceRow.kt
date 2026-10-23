package dev.neurokitchen.quire.presentation.components.reference_card

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import org.jetbrains.compose.resources.painterResource

/**
 * High-density compact row for the Library tab:
 * - Optimized for browsing large document collections (height ~62dp).
 * - 36x36dp thumbnail anchor (tapping directly opens reader if PDF present).
 * - Two-line concise typographic hierarchy with inline status dot.
 * - Tactile rounded surface with zero harsh borders.
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun LibraryReferenceRow(
    state: ReferenceCardState,
    onEvent: (ReferenceCardEvent) -> Unit,
    modifier: Modifier = Modifier,
    action: (@Composable () -> Unit)? = null,
) {
    val colors = QuireTheme.colors

    Card(
        colors = CardDefaults.cardColors(
            containerColor = if (state.isSelected) colors.accent.copy(alpha = 0.16f) else colors.cardBackground,
        ),
        shape = RoundedCornerShape(14.dp),
        border = null,
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .combinedClickable(
                onClick = { onEvent(ReferenceCardEvent.OnClick(state.id)) },
                onLongClick = { onEvent(ReferenceCardEvent.OnLongClick(state)) },
            ),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 12.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            // Visual Anchor: Compact 36x36 thumbnail icon
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(RoundedCornerShape(10.dp))
                    .background(
                        if (state.hasPdf) colors.accent.copy(alpha = 0.14f)
                        else colors.fillQuinary,
                    )
                    .clickable {
                        onEvent(ReferenceCardEvent.OnOpen(state.id))
                    },
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    painter = painterResource(
                        if (state.hasPdf) Res.drawable.ic_picture_as_pdf
                        else Res.drawable.ic_description,
                    ),
                    contentDescription = if (state.hasPdf) "Open PDF" else null,
                    tint = if (state.hasPdf) colors.accent else colors.fillSecondary,
                    modifier = Modifier.size(18.dp),
                )
            }

            Spacer(Modifier.width(12.dp))

            // Main info: Title + Meta in two clean lines
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = state.title,
                    style = MaterialTheme.typography.titleSmall.copy(
                        fontFamily = FontFamily.Serif,
                        fontWeight = FontWeight.SemiBold,
                        fontSize = 14.sp,
                        lineHeight = 19.sp,
                    ),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    color = colors.fillPrimary,
                )

                Spacer(Modifier.height(2.dp))

                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    val meta = listOf(state.creator, state.year).filter { it.isNotEmpty() }.joinToString(" · ")
                    if (meta.isNotEmpty()) {
                        Text(
                            text = meta,
                            style = MaterialTheme.typography.bodySmall.copy(fontSize = 12.sp),
                            color = colors.fillSecondary,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f, fill = false),
                        )
                    }

                    // Compact Reading Status Dot
                    if (state.reading != ReadingStatus.Unread) {
                        Box(
                            modifier = Modifier
                                .size(5.dp)
                                .clip(CircleShape)
                                .background(
                                    if (state.reading == ReadingStatus.Reading) colors.statusReading
                                    else colors.statusRead,
                                ),
                        )
                        Text(
                            text = if (state.reading == ReadingStatus.Reading) "Reading" else "Read",
                            style = MaterialTheme.typography.labelSmall.copy(
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Medium,
                            ),
                            color = if (state.reading == ReadingStatus.Reading) colors.statusReading else colors.statusRead,
                        )
                    }
                }
            }

            if (action != null) {
                Spacer(Modifier.width(8.dp))
                action()
            }
        }
    }
}
