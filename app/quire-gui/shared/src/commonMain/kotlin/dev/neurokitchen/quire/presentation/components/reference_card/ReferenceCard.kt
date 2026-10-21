package dev.neurokitchen.quire.presentation.components.reference_card

import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
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
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import org.jetbrains.compose.resources.painterResource

/**
 * Premium tactile reference card inspired by Pocket Hoot & modern document readers:
 * - NO stroke borders / harsh edges
 * - 20dp smooth corner radius with distinct surface tone
 * - Stylized document thumbnail icon container on the left
 * - Clear typographic hierarchy and reading status badge
 */
@Composable
fun ReferenceCard(
    state: ReferenceCardState,
    onEvent: (ReferenceCardEvent) -> Unit,
    modifier: Modifier = Modifier,
    /**
     * `false` (phones): no cite key over the title, the reading status under it instead, beside
     * [action] in the bottom corner (Cite).
     */
    showCiteKey: Boolean = true,
    action: (@Composable () -> Unit)? = null,
) {
    val colors = QuireTheme.colors
    Card(
        onClick = { onEvent(ReferenceCardEvent.OnClick(state.id)) },
        colors = CardDefaults.cardColors(
            containerColor = if (state.isSelected) colors.accent.copy(alpha = 0.16f) else colors.cardBackground,
        ),
        shape = RoundedCornerShape(20.dp),
        // No stroke border (edges eliminated)
        border = null,
        modifier = modifier.fillMaxWidth(),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.Top,
        ) {
            // Visual Anchor: Stylized 46x46 Rounded Thumbnail Box on the left
            Box(
                modifier = Modifier
                    .size(46.dp)
                    .clip(RoundedCornerShape(14.dp))
                    .background(
                        if (state.hasPdf) colors.accent.copy(alpha = 0.15f)
                        else colors.fillQuinary,
                    ),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    painter = painterResource(
                        if (state.hasPdf) Res.drawable.ic_picture_as_pdf
                        else Res.drawable.ic_description,
                    ),
                    contentDescription = null,
                    tint = if (state.hasPdf) colors.accent else colors.fillSecondary,
                    modifier = Modifier.size(22.dp),
                )
            }

            Spacer(Modifier.width(14.dp))

            // Main Details
            Column(modifier = Modifier.weight(1f)) {
                // Top Meta: CiteKey + Reading Status
                if (showCiteKey) Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    if (showCiteKey && state.citeKey.isNotEmpty()) {
                        Text(
                            text = state.citeKey,
                            style = MaterialTheme.typography.labelSmall.copy(
                                fontFamily = FontFamily.Monospace,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Medium,
                            ),
                            color = colors.fillSecondary,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    } else {
                        Spacer(Modifier.width(1.dp))
                    }

                    ReadingPill(state.reading)
                }

                if (showCiteKey) Spacer(Modifier.height(6.dp))

                // Title
                Text(
                    text = state.title,
                    style = MaterialTheme.typography.titleMedium.copy(
                        fontWeight = FontWeight.Bold,
                        fontSize = 15.sp,
                        lineHeight = 21.sp,
                    ),
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    color = colors.fillPrimary,
                )

                Spacer(Modifier.height(4.dp))

                // Author and Year
                Text(
                    text = listOf(state.creator, state.year).filter { it.isNotEmpty() }.joinToString(" · "),
                    style = MaterialTheme.typography.bodySmall.copy(fontSize = 13.sp),
                    color = colors.fillSecondary,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (!showCiteKey) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier.fillMaxWidth().padding(top = Dimens.SpaceXs).heightIn(min = 36.dp),
                    ) {
                        ReadingPill(state.reading)
                        Spacer(Modifier.weight(1f))
                        action?.invoke()
                    }
                }
            }
        }
    }
}

/** The reading status: a dot and a word, on a pill of its colour. */
@Composable
private fun ReadingPill(reading: ReadingStatus) {
    val colors = QuireTheme.colors
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .clip(RoundedCornerShape(Dimens.RadiusFull))
            .background(
                when (reading) {
                    ReadingStatus.Read -> colors.statusRead.copy(alpha = 0.15f)
                    ReadingStatus.Reading -> colors.statusReading.copy(alpha = 0.15f)
                    ReadingStatus.Unread -> colors.fillQuinary
                },
            )
            .padding(horizontal = 8.dp, vertical = 3.dp),
    ) {
        Box(
            modifier = Modifier
                .size(6.dp)
                .clip(CircleShape)
                .background(
                    when (reading) {
                        ReadingStatus.Read -> colors.statusRead
                        ReadingStatus.Reading -> colors.statusReading
                        ReadingStatus.Unread -> colors.fillSecondary
                    },
                ),
        )
        Spacer(Modifier.width(5.dp))
        Text(
            text = when (reading) {
                ReadingStatus.Read -> "Read"
                ReadingStatus.Reading -> "Reading"
                ReadingStatus.Unread -> "Unread"
            },
            style = MaterialTheme.typography.labelSmall.copy(
                fontSize = 11.sp,
                fontWeight = FontWeight.SemiBold,
            ),
            color = when (reading) {
                ReadingStatus.Read -> colors.statusRead
                ReadingStatus.Reading -> colors.statusReading
                ReadingStatus.Unread -> colors.fillSecondary
            },
        )
    }
}

