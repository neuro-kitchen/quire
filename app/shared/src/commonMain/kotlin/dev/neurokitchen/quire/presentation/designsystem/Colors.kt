package dev.neurokitchen.quire.presentation.designsystem

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color

/**
 * The app's color tokens. For now they are Zotero 7's (`scss/themes/_light.scss`, `_dark.scss`), so
 * the desktop app reads like Zotero before quire gets a look of its own.
 */
@Immutable
data class QuireColors(
    val background: Color,
    val sidepane: Color,
    val toolbar: Color,
    val tabbar: Color,
    val paneDivider: Color,
    val border: Color,
    /** Every other row of a table. */
    val stripe: Color,
    val accent: Color,
    /** PDFs, errors. */
    val accentRed: Color,
    val onAccent: Color,
    /** A selected row in a pane without focus. */
    val selectedInactive: Color,
    val fillPrimary: Color,
    val fillSecondary: Color,
    val fillTertiary: Color,
    val fillQuarternary: Color,
    val fillQuinary: Color,
    val sectionInfo: Color,
    val sectionAbstract: Color,
    val sectionAttachments: Color,
    val sectionNotes: Color,
    val sectionProjects: Color,
    val sectionTags: Color,
    val sectionRelated: Color,
    val isDark: Boolean,
)

val LightQuireColors = QuireColors(
    background = Color(0xFFFFFFFF),
    sidepane = Color(0xFFF2F2F2),
    toolbar = Color(0xFFF9F9F9),
    tabbar = Color(0xFFF2F2F2),
    paneDivider = Color(0xFFDADADA),
    border = Color(0x26000000),
    stripe = Color(0x0A000000),
    accent = Color(0xFF4072E5),
    accentRed = Color(0xFFDB2C3A),
    onAccent = Color(0xFFFFFFFF),
    selectedInactive = Color(0x1A000000),
    fillPrimary = Color(0xD9000000),
    fillSecondary = Color(0x8C000000),
    fillTertiary = Color(0x40000000),
    fillQuarternary = Color(0x1A000000),
    fillQuinary = Color(0x0D000000),
    sectionInfo = Color(0xFF4072E5),
    sectionAbstract = Color(0xFF66ADFF),
    sectionAttachments = Color(0xFF39BF68),
    sectionNotes = Color(0xFFFAA700),
    sectionProjects = Color(0xFF59ADC4),
    sectionTags = Color(0xFFFF794C),
    sectionRelated = Color(0xFFCC7A52),
    isDark = false,
)

val DarkQuireColors = QuireColors(
    background = Color(0xFF1E1E1E),
    sidepane = Color(0xFF303030),
    toolbar = Color(0xFF272727),
    tabbar = Color(0xFF1E1E1E),
    paneDivider = Color(0xFF404040),
    border = Color(0x2EFFFFFF),
    stripe = Color(0x0DFFFFFF),
    accent = Color(0xFF4072E5),
    accentRed = Color(0xE5DB2C3A),
    onAccent = Color(0xFFFFFFFF),
    selectedInactive = Color(0x1FFFFFFF),
    fillPrimary = Color(0xE5FFFFFF),
    fillSecondary = Color(0x8CFFFFFF),
    fillTertiary = Color(0x4DFFFFFF),
    fillQuarternary = Color(0x1FFFFFFF),
    fillQuinary = Color(0x0FFFFFFF),
    sectionInfo = Color(0xFF4072E5),
    sectionAbstract = Color(0xD966ADFF),
    sectionAttachments = Color(0xD939BF68),
    sectionNotes = Color(0xCCFAA700),
    sectionProjects = Color(0xE559ADC4),
    sectionTags = Color(0xD9FF794C),
    sectionRelated = Color(0xE5CC7A52),
    isDark = true,
)
