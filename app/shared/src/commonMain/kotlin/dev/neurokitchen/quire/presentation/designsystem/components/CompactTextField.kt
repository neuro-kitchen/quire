package dev.neurokitchen.quire.presentation.designsystem.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource

/** A single-line 28 px field for toolbars and filters, with an optional leading icon. */
@Composable
fun CompactTextField(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    modifier: Modifier = Modifier,
    leadingIcon: DrawableResource? = null,
) {
    val colors = QuireTheme.colors
    val textStyle = MaterialTheme.typography.bodyMedium.copy(color = colors.fillPrimary)
    BasicTextField(
        value = value,
        onValueChange = onValueChange,
        singleLine = true,
        textStyle = textStyle,
        cursorBrush = SolidColor(colors.accent),
        modifier = modifier,
        decorationBox = { inner ->
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier
                    .height(Dimens.ToolbarButton)
                    .background(colors.background, RoundedCornerShape(Dimens.RadiusLarge))
                    .border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusLarge))
                    .padding(horizontal = 6.dp),
            ) {
                if (leadingIcon != null) {
                    Icon(painterResource(leadingIcon), null, tint = colors.fillSecondary, modifier = Modifier.size(Dimens.Icon))
                    Box(Modifier.width(6.dp))
                }
                Box(Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
                    if (value.isEmpty()) Text(placeholder, style = textStyle.copy(color = colors.fillTertiary), maxLines = 1)
                    inner()
                }
            }
        },
    )
}
