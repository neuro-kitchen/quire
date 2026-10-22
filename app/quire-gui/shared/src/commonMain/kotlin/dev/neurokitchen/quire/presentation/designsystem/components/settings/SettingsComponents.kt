package dev.neurokitchen.quire.presentation.designsystem.components.settings

import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_chevron_right
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource

/*
 * Settings in the style of Material 3 Expressive, from stable Material 3 (its expressive components
 * are alpha-only, see `.knowledge/app.md`): a large header; segmented groups — each row its own
 * rounded block, large corners at the group's ends and small ones between; a leading icon in a
 * tinted circle; generous rows; a spring on press. Phone and desktop share them.
 */

/** Corner of a group's ends, and of the joins between its rows. */
private val OuterCorner = 24.dp
private val InnerCorner = 6.dp

/** The gap between a group's rows (the "segments"). */
private val SegmentGap = 2.dp

/** Row [index] of [count] in a segmented group: rounded ends, slight joins between. */
fun segmentedShape(index: Int, count: Int): Shape {
    val top = if (index == 0) OuterCorner else InnerCorner
    val bottom = if (index == count - 1) OuterCorner else InnerCorner
    return RoundedCornerShape(topStart = top, topEnd = top, bottomStart = bottom, bottomEnd = bottom)
}

/** A page's title, large, and what the page is for. */
@Composable
fun SettingsHeader(
    title: String,
    subtitle: String? = null,
    modifier: Modifier = Modifier,
    horizontalPadding: Dp = Dimens.SpaceLg,
) {
    Column(modifier.fillMaxWidth().padding(horizontal = horizontalPadding, vertical = Dimens.SpaceMd)) {
        Text(title, style = MaterialTheme.typography.headlineLarge, fontWeight = FontWeight.Bold, color = QuireTheme.colors.fillPrimary)
        if (subtitle != null) {
            Spacer(Modifier.size(Dimens.SpaceMin))
            Text(subtitle, style = MaterialTheme.typography.bodyLarge, color = QuireTheme.colors.fillSecondary)
        }
    }
}

/**
 * A group of rows under an optional [label]: each of [rows] gets its segment's shape (pass it to
 * [SettingsItem] or clip with it).
 */
@Composable
fun SettingsGroup(
    rows: List<@Composable (shape: Shape) -> Unit>,
    modifier: Modifier = Modifier,
    label: String? = null,
    horizontalPadding: Dp = Dimens.SpaceMd,
) {
    Column(modifier.fillMaxWidth().padding(horizontal = horizontalPadding)) {
        if (label != null) {
            Text(
                label,
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.SemiBold,
                color = QuireTheme.colors.accent,
                modifier = Modifier.padding(start = Dimens.SpaceSm, top = Dimens.SpaceMd, bottom = Dimens.SpaceXs),
            )
        }
        Column(verticalArrangement = Arrangement.spacedBy(SegmentGap)) {
            rows.forEachIndexed { index, row -> row(segmentedShape(index, rows.size)) }
        }
    }
}

/**
 * One row of a group: a tinted [icon] (or [leading] content), [title] and [supporting] text, and a
 * [trailing] part (a chevron when the row opens a page: [opensPage]). Pressing it springs.
 */
@Composable
fun SettingsItem(
    title: String,
    shape: Shape,
    modifier: Modifier = Modifier,
    supporting: String? = null,
    icon: DrawableResource? = null,
    iconTint: Color = QuireTheme.colors.accent,
    leading: (@Composable () -> Unit)? = null,
    opensPage: Boolean = false,
    enabled: Boolean = true,
    onClick: (() -> Unit)? = null,
    trailing: (@Composable RowScope.() -> Unit)? = null,
) {
    val colors = QuireTheme.colors
    val interaction = remember { MutableInteractionSource() }
    val pressed by interaction.collectIsPressedAsState()
    // A slight, springy give under the finger (Expressive motion, with stable APIs).
    val scale by animateFloatAsState(
        targetValue = if (pressed) 0.97f else 1f,
        animationSpec = spring(dampingRatio = Spring.DampingRatioMediumBouncy, stiffness = Spring.StiffnessMediumLow),
        label = "press",
    )
    Row(
        modifier
            .fillMaxWidth()
            .graphicsLayer { scaleX = scale; scaleY = scale }
            .clip(shape)
            .background(colors.cardBackground)
            .then(
                if (onClick != null) {
                    Modifier.clickable(interactionSource = interaction, indication = androidx.compose.material3.ripple(), enabled = enabled, role = Role.Button, onClick = onClick)
                } else {
                    Modifier
                },
            )
            .heightIn(min = 72.dp)
            .padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceSm),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        when {
            leading != null -> {
                leading()
                Spacer(Modifier.width(Dimens.SpaceMd))
            }
            icon != null -> {
                Box(
                    Modifier.size(40.dp).clip(CircleShape).background(iconTint.copy(alpha = 0.16f)),
                    contentAlignment = Alignment.Center,
                ) { Icon(painterResource(icon), contentDescription = null, tint = iconTint, modifier = Modifier.size(22.dp)) }
                Spacer(Modifier.width(Dimens.SpaceMd))
            }
        }
        Column(Modifier.weight(1f)) {
            Text(
                title,
                style = MaterialTheme.typography.titleMedium,
                color = if (enabled) colors.fillPrimary else colors.fillTertiary,
            )
            if (supporting != null) {
                Text(supporting, style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary)
            }
        }
        if (trailing != null) {
            Spacer(Modifier.width(Dimens.SpaceSm))
            trailing()
        }
        if (opensPage) {
            Spacer(Modifier.width(Dimens.SpaceXs))
            Icon(painterResource(Res.drawable.ic_chevron_right), contentDescription = null, tint = colors.fillSecondary)
        }
    }
}

/** A count to notice beside a row (unread notifications): white on red, in a pill. */
@Composable
fun SettingsBadge(text: String) {
    Text(
        text,
        style = MaterialTheme.typography.labelMedium,
        fontWeight = FontWeight.SemiBold,
        color = QuireTheme.colors.onAccent,
        modifier = Modifier.clip(CircleShape).background(QuireTheme.colors.accentRed).padding(horizontal = 8.dp, vertical = 2.dp),
    )
}
