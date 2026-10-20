package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_search
import org.jetbrains.compose.resources.painterResource

/**
 * TUI-parity In-Reader Search Bar:
 * Allows searching within PDF text and document sections with match counter and navigation.
 */
@Composable
fun ReaderSearchBar(
    query: String,
    onQueryChange: (String) -> Unit,
    matchCount: Int = 0,
    currentMatch: Int = 0,
    onNextMatch: () -> Unit = {},
    onPrevMatch: () -> Unit = {},
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    val focusRequester = remember { FocusRequester() }

    LaunchedEffect(Unit) {
        focusRequester.requestFocus()
    }

    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        modifier = modifier
            .fillMaxWidth()
            .height(38.dp)
            .background(colors.sidepane)
            .border(1.dp, colors.border, RoundedCornerShape(0.dp))
            .padding(horizontal = Dimens.SpaceMd),
    ) {
        Icon(
            painterResource(Res.drawable.ic_search),
            contentDescription = null,
            tint = colors.accent,
            modifier = Modifier.size(16.dp),
        )

        Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
            if (query.isEmpty()) {
                Text(
                    "Find in document...",
                    style = MaterialTheme.typography.bodyMedium,
                    color = colors.fillTertiary,
                )
            }
            BasicTextField(
                value = query,
                onValueChange = onQueryChange,
                textStyle = MaterialTheme.typography.bodyMedium.copy(color = colors.fillPrimary),
                cursorBrush = SolidColor(colors.accent),
                singleLine = true,
                modifier = Modifier
                    .fillMaxWidth()
                    .focusRequester(focusRequester),
            )
        }

        if (query.isNotEmpty()) {
            Text(
                text = if (matchCount > 0) "$currentMatch of $matchCount" else "0 matches",
                style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp),
                color = if (matchCount > 0) colors.fillSecondary else colors.accentRed,
            )

            // Previous match
            Box(
                modifier = Modifier
                    .size(24.dp)
                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                    .background(colors.fillQuarternary)
                    .clickable(enabled = matchCount > 0, onClick = onPrevMatch),
                contentAlignment = Alignment.Center,
            ) {
                Text("↑", style = MaterialTheme.typography.labelSmall, color = colors.fillPrimary)
            }

            // Next match
            Box(
                modifier = Modifier
                    .size(24.dp)
                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                    .background(colors.fillQuarternary)
                    .clickable(enabled = matchCount > 0, onClick = onNextMatch),
                contentAlignment = Alignment.Center,
            ) {
                Text("↓", style = MaterialTheme.typography.labelSmall, color = colors.fillPrimary)
            }
        }

        Spacer(Modifier.width(4.dp))

        // Close button
        Box(
            modifier = Modifier
                .size(24.dp)
                .clip(RoundedCornerShape(Dimens.RadiusSmall))
                .clickable(onClick = onClose),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                painterResource(Res.drawable.ic_close),
                contentDescription = "Close Search (Esc)",
                tint = colors.fillSecondary,
                modifier = Modifier.size(14.dp),
            )
        }
    }
}
