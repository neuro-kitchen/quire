package dev.neurokitchen.quire.presentation.screens.reader.components

import dev.neurokitchen.quire.resources.reader_open
import dev.neurokitchen.quire.resources.reader_close_pane
import dev.neurokitchen.quire.resources.common_copy_citekey
import dev.neurokitchen.quire.resources.citation_in_library
import dev.neurokitchen.quire.resources.citation_copy_key
import org.jetbrains.compose.resources.stringResource
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
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_content_copy
import dev.neurokitchen.quire.resources.ic_description
import org.jetbrains.compose.resources.painterResource

data class CitationDetail(
    val key: String,
    val title: String,
    val authors: String,
    val year: String,
    val venue: String? = null,
    val abstract: String? = null,
    val inLibrary: Boolean = false,
)

@Composable
fun CitationHoverCard(
    citation: CitationDetail?,
    onDismissRequest: () -> Unit,
    onOpenPaper: ((String) -> Unit)? = null,
) {
    if (citation == null) return

    val colors = QuireTheme.colors
    val clipboard = LocalClipboardManager.current

    Dialog(onDismissRequest = onDismissRequest) {
        Surface(
            shape = RoundedCornerShape(Dimens.RadiusBase),
            color = colors.sidepane,
            shadowElevation = 16.dp,
            modifier = Modifier
                .width(420.dp)
                .border(1.dp, colors.border.copy(alpha = 0.8f), RoundedCornerShape(Dimens.RadiusBase)),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(Dimens.SpaceMd),
            ) {
                // Header Bar
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                .background(colors.accent.copy(alpha = 0.15f))
                                .padding(horizontal = Dimens.SpaceXs, vertical = 3.dp),
                        ) {
                            Text(
                                "@${citation.key}",
                                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold, fontSize = 11.sp),
                                color = colors.accent,
                            )
                        }
                        if (citation.inLibrary) {
                            Spacer(Modifier.width(6.dp))
                            Box(
                                modifier = Modifier
                                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                    .background(colors.statusRead.copy(alpha = 0.15f))
                                    .padding(horizontal = 6.dp, vertical = 3.dp),
                            ) {
                                Text(
                                    stringResource(Res.string.citation_in_library),
                                    style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold, fontSize = 9.sp),
                                    color = colors.statusRead,
                                )
                            }
                        }
                    }

                    Box(
                        contentAlignment = Alignment.Center,
                        modifier = Modifier
                            .size(24.dp)
                            .clip(CircleShape)
                            .background(colors.fillQuarternary)
                            .clickable(onClick = onDismissRequest),
                    ) {
                        Icon(
                            painterResource(Res.drawable.ic_close),
                            contentDescription = stringResource(Res.string.reader_close_pane),
                            tint = colors.fillSecondary,
                            modifier = Modifier.size(14.dp),
                        )
                    }
                }

                Spacer(Modifier.height(10.dp))

                // Title
                Text(
                    text = citation.title,
                    style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.SemiBold, fontSize = 14.sp),
                    color = colors.fillPrimary,
                )

                Spacer(Modifier.height(6.dp))

                // Authors & Year
                Text(
                    text = buildString {
                        append(citation.authors)
                        if (citation.year.isNotBlank()) append(" (${citation.year})")
                        if (!citation.venue.isNullOrBlank()) append(" • ${citation.venue}")
                    },
                    style = MaterialTheme.typography.bodySmall.copy(fontSize = 11.sp),
                    color = colors.fillSecondary,
                )

                // Abstract snippet
                if (!citation.abstract.isNullOrBlank()) {
                    Spacer(Modifier.height(10.dp))
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(Dimens.RadiusSmall))
                            .background(colors.background.copy(alpha = 0.6f))
                            .border(1.dp, colors.border.copy(alpha = 0.3f), RoundedCornerShape(Dimens.RadiusSmall))
                            .padding(Dimens.SpaceXs),
                    ) {
                        Text(
                            text = citation.abstract,
                            style = MaterialTheme.typography.bodySmall.copy(fontSize = 11.sp, lineHeight = 15.sp),
                            color = colors.fillPrimary.copy(alpha = 0.85f),
                            maxLines = 4,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }

                Spacer(Modifier.height(14.dp))
                HorizontalDivider(color = colors.border.copy(alpha = 0.4f))
                Spacer(Modifier.height(10.dp))

                // Action Bar
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier
                            .clip(RoundedCornerShape(Dimens.RadiusSmall))
                            .background(colors.fillQuarternary.copy(alpha = 0.6f))
                            .clickable { clipboard.setText(AnnotatedString("@${citation.key}")) }
                            .padding(horizontal = Dimens.SpaceXs, vertical = Dimens.SpaceMin),
                    ) {
                        Icon(
                            painterResource(Res.drawable.ic_content_copy),
                            contentDescription = stringResource(Res.string.common_copy_citekey),
                            tint = colors.fillSecondary,
                            modifier = Modifier.size(12.dp),
                        )
                        Spacer(Modifier.width(Dimens.SpaceMin))
                        Text(
                            stringResource(Res.string.citation_copy_key),
                            style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp),
                            color = colors.fillSecondary,
                        )
                    }

                    if (citation.inLibrary && onOpenPaper != null) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                .background(colors.accent)
                                .clickable {
                                    onDismissRequest()
                                    onOpenPaper(citation.key)
                                }
                                .padding(horizontal = 10.dp, vertical = 5.dp),
                        ) {
                            Text(
                                stringResource(Res.string.reader_open),
                                style = MaterialTheme.typography.labelSmall.copy(
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 11.sp,
                                ),
                                color = colors.onAccent,
                            )
                        }
                    }
                }
            }
        }
    }
}
