package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.library_search_hint
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

@Composable
fun LibrarySearchField(
    query: String,
    onQueryChange: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    OutlinedTextField(
        value = query,
        onValueChange = onQueryChange,
        placeholder = {
            Text(
                stringResource(Res.string.library_search_hint),
                style = MaterialTheme.typography.bodyMedium,
                color = colors.fillSecondary,
            )
        },
        leadingIcon = {
            Icon(
                painter = painterResource(Res.drawable.ic_search),
                contentDescription = "Search",
                tint = colors.fillSecondary,
                modifier = Modifier.size(20.dp),
            )
        },
        trailingIcon = {
            if (query.isNotEmpty()) {
                Icon(
                    painter = painterResource(Res.drawable.ic_close),
                    contentDescription = "Clear search",
                    tint = colors.fillSecondary,
                    modifier = Modifier
                        .size(18.dp)
                        .clip(RoundedCornerShape(Dimens.RadiusFull))
                        .clickable { onQueryChange("") },
                )
            }
        },
        singleLine = true,
        shape = RoundedCornerShape(14.dp),
        colors = OutlinedTextFieldDefaults.colors(
            focusedContainerColor = colors.cardBackground,
            unfocusedContainerColor = colors.cardBackground,
            focusedBorderColor = colors.accent,
            unfocusedBorderColor = colors.border.copy(alpha = 0.45f),
            cursorColor = colors.accent,
            focusedTextColor = colors.fillPrimary,
            unfocusedTextColor = colors.fillPrimary,
        ),
        textStyle = MaterialTheme.typography.bodyMedium,
        modifier = modifier.fillMaxWidth(),
    )
}
