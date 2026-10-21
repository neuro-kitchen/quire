package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.ReferenceDetailState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_arrow_back
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/**
 * Mobile Reference Detail Screen:
 * - Bib-style citation pill in the top header
 * - Title, Authors, and Date
 * - Reading status indicator pill directly below authors and date
 * - Scrollable abstract and metadata
 * - Fixed bottom action bar with thumb-accessible "Read Document" hero button
 */
@Composable
fun ReferenceDetailPane(
    reference: ReferenceDetailState,
    onClose: () -> Unit,
    onOpen: () -> Unit = {},
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors

    Box(
        modifier = modifier
            .fillMaxSize()
            .background(colors.background),
    ) {
        // Scrollable Content
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 20.dp, vertical = 16.dp)
                .padding(bottom = 84.dp),
        ) {
            // Header: Back button & Bib citation key pill
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier.fillMaxWidth().padding(bottom = 12.dp),
            ) {
                Box(
                    modifier = Modifier
                        .size(40.dp)
                        .clip(CircleShape)
                        .background(colors.cardBackground)
                        .clickable(onClick = onClose),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_arrow_back),
                        contentDescription = "Back to Library",
                        tint = colors.fillPrimary,
                        modifier = Modifier.size(20.dp),
                    )
                }

                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(Dimens.RadiusFull))
                        .background(colors.cardBackground)
                        .padding(horizontal = 14.dp, vertical = 6.dp),
                ) {
                    Text(
                        text = reference.citeKey.ifEmpty { "no-key" },
                        style = MaterialTheme.typography.labelSmall.copy(
                            fontFamily = FontFamily.Monospace,
                            fontWeight = FontWeight.SemiBold,
                            fontSize = 12.sp,
                        ),
                        color = colors.accent,
                    )
                }
            }

            Spacer(Modifier.height(10.dp))

            // Full Title
            Text(
                text = reference.title,
                style = MaterialTheme.typography.headlineSmall.copy(
                    fontWeight = FontWeight.Bold,
                    lineHeight = 30.sp,
                    fontSize = 22.sp,
                ),
                color = colors.fillPrimary,
            )

            Spacer(Modifier.height(8.dp))

            // Authors & Year
            Text(
                text = listOf(reference.creator, reference.year).filter { it.isNotEmpty() }.joinToString(" · "),
                style = MaterialTheme.typography.bodyLarge.copy(fontSize = 15.sp),
                color = colors.fillSecondary,
            )

            Spacer(Modifier.height(14.dp))

            // Reading Status Pill directly below Title, Authors, and Date
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier
                    .clip(RoundedCornerShape(Dimens.RadiusFull))
                    .background(
                        when (reference.reading) {
                            ReadingStatus.Read -> colors.statusRead.copy(alpha = 0.15f)
                            ReadingStatus.Reading -> colors.statusReading.copy(alpha = 0.15f)
                            ReadingStatus.Unread -> colors.fillQuinary
                        },
                    )
                    .padding(horizontal = 12.dp, vertical = 6.dp),
            ) {
                Box(
                    modifier = Modifier
                        .size(8.dp)
                        .clip(CircleShape)
                        .background(
                            when (reference.reading) {
                                ReadingStatus.Read -> colors.statusRead
                                ReadingStatus.Reading -> colors.statusReading
                                ReadingStatus.Unread -> colors.fillSecondary
                            },
                        ),
                )
                Spacer(Modifier.width(8.dp))
                Text(
                    text = when (reference.reading) {
                        ReadingStatus.Read -> "Read"
                        ReadingStatus.Reading -> "Reading"
                        ReadingStatus.Unread -> "Unread"
                    },
                    style = MaterialTheme.typography.labelSmall.copy(
                        fontSize = 13.sp,
                        fontWeight = FontWeight.SemiBold,
                    ),
                    color = when (reference.reading) {
                        ReadingStatus.Read -> colors.statusRead
                        ReadingStatus.Reading -> colors.statusReading
                        ReadingStatus.Unread -> colors.fillSecondary
                    },
                )
            }

            // Abstract section
            if (reference.abstract.isNotEmpty()) {
                Spacer(Modifier.height(20.dp))
                Text(
                    "Abstract",
                    style = MaterialTheme.typography.labelMedium.copy(
                        fontWeight = FontWeight.Bold,
                        color = colors.fillSecondary,
                        fontSize = 12.sp,
                    ),
                )
                Spacer(Modifier.height(6.dp))
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(14.dp))
                        .background(colors.cardBackground)
                        .padding(16.dp),
                ) {
                    Text(
                        text = reference.abstract,
                        style = MaterialTheme.typography.bodyMedium.copy(lineHeight = 22.sp, fontSize = 14.sp),
                        color = colors.fillPrimary,
                    )
                }
            }

            // Additional metadata if available
            if (reference.fields.isNotEmpty()) {
                Spacer(Modifier.height(20.dp))
                Text(
                    "Details",
                    style = MaterialTheme.typography.labelMedium.copy(
                        fontWeight = FontWeight.Bold,
                        color = colors.fillSecondary,
                        fontSize = 12.sp,
                    ),
                )
                Spacer(Modifier.height(6.dp))
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(14.dp))
                        .background(colors.cardBackground)
                        .padding(horizontal = 16.dp, vertical = 10.dp),
                ) {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        reference.fields.forEach { field ->
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                            ) {
                                Text(
                                    text = stringResource(field.label),
                                    style = MaterialTheme.typography.bodySmall.copy(fontSize = 13.sp),
                                    color = colors.fillSecondary,
                                    modifier = Modifier.weight(1f),
                                )
                                Text(
                                    text = field.value,
                                    style = MaterialTheme.typography.bodySmall.copy(
                                        fontSize = 13.sp,
                                        fontWeight = FontWeight.Medium,
                                    ),
                                    color = colors.fillPrimary,
                                    modifier = Modifier.weight(2f),
                                )
                            }
                        }
                    }
                }
            }
        }

        // Fixed Bottom Action Bar for Thumb-Accessible "Read Document"
        Box(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .background(colors.background)
                .padding(horizontal = 20.dp, vertical = 12.dp)
                .navigationBarsPadding(),
        ) {
            Button(
                onClick = onOpen,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(52.dp),
                colors = ButtonDefaults.buttonColors(containerColor = colors.accent),
                shape = RoundedCornerShape(16.dp),
            ) {
                Icon(
                    painterResource(Res.drawable.ic_picture_as_pdf),
                    contentDescription = null,
                    tint = colors.onAccent,
                    modifier = Modifier.size(20.dp),
                )
                Spacer(Modifier.width(10.dp))
                Text(
                    "Read Document",
                    style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Bold, fontSize = 15.sp),
                    color = colors.onAccent,
                )
            }
        }
    }
}
