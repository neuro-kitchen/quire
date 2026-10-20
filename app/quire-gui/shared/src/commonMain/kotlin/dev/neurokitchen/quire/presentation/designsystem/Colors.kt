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
    val surfaceContainerLowest: Color = if (isDark) Color(0xFF121212) else Color(0xFFFFFFFF),
    val surfaceContainerLow: Color = if (isDark) Color(0xFF1E1E1E) else Color(0xFFF8FAFC),
    val surfaceContainer: Color = if (isDark) Color(0xFF262626) else Color(0xFFF1F5F9),
    val surfaceContainerHigh: Color = if (isDark) Color(0xFF2E3135) else Color(0xFFE2E8F0),
    val surfaceContainerHighest: Color = if (isDark) Color(0xFF3B3E43) else Color(0xFFCBD5E1),
    val highlightYellow: Color = if (isDark) Color(0x66FBBF24) else Color(0xFFFEF08A),
    val highlightGreen: Color = if (isDark) Color(0x6634D399) else Color(0xFFBBF7D0),
    val highlightBlue: Color = if (isDark) Color(0x6638BDF8) else Color(0xFFBAE6FD),
    val highlightPurple: Color = if (isDark) Color(0x66C084FC) else Color(0xFFE9D5FF),
    val highlightRed: Color = if (isDark) Color(0x66FB7185) else Color(0xFFFECDD3),
    val statusRead: Color = Color(0xFF10B981),
    val statusReading: Color = Color(0xFFF59E0B),
    val statusUnread: Color = Color(0xFF94A3B8),
)

val LightQuireColors = QuireColors(
    background = Color(0xFFFFFFFF),
    sidepane = Color(0xFFF8FAFC),
    toolbar = Color(0xFFFFFFFF),
    tabbar = Color(0xFFF1F5F9),
    paneDivider = Color(0xFFE2E8F0),
    border = Color(0x1F000000),
    stripe = Color(0x06000000),
    accent = Color(0xFF4F46E5),
    accentRed = Color(0xFFE11D48),
    onAccent = Color(0xFFFFFFFF),
    selectedInactive = Color(0x14000000),
    fillPrimary = Color(0xD90F172A),
    fillSecondary = Color(0x8C0F172A),
    fillTertiary = Color(0x400F172A),
    fillQuarternary = Color(0x140F172A),
    fillQuinary = Color(0x0A0F172A),
    sectionInfo = Color(0xFF4F46E5),
    sectionAbstract = Color(0xFF0284C7),
    sectionAttachments = Color(0xFF059669),
    sectionNotes = Color(0xFFD97706),
    sectionProjects = Color(0xFF0891B2),
    sectionTags = Color(0xFFEA580C),
    sectionRelated = Color(0xFF9333EA),
    isDark = false,
)

val DarkQuireColors = QuireColors(
    background = Color(0xFF121316),
    sidepane = Color(0xFF18191E),
    toolbar = Color(0xFF1F2128),
    tabbar = Color(0xFF15161A),
    paneDivider = Color(0xFF282A32),
    border = Color(0x2EFFFFFF),
    stripe = Color(0x08FFFFFF),
    accent = Color(0xFF6366F1),
    accentRed = Color(0xFFF43F5E),
    onAccent = Color(0xFFFFFFFF),
    selectedInactive = Color(0x1AFFFFFF),
    fillPrimary = Color(0xF2F8FAFC),
    fillSecondary = Color(0x8CF8FAFC),
    fillTertiary = Color(0x4DF8FAFC),
    fillQuarternary = Color(0x1AF8FAFC),
    fillQuinary = Color(0x0CF8FAFC),
    sectionInfo = Color(0xFF818CF8),
    sectionAbstract = Color(0xFF38BDF8),
    sectionAttachments = Color(0xFF34D399),
    sectionNotes = Color(0xFFFBBF24),
    sectionProjects = Color(0xFF22D3EE),
    sectionTags = Color(0xFFFB923C),
    sectionRelated = Color(0xFFC084FC),
    isDark = true,
)

enum class AppTheme(val title: String, val isDark: Boolean) {
    Obsidian("Obsidian Dark", isDark = true),
    Nord("Nord Arctic", isDark = true),
    Solarized("Solarized Dark", isDark = true),
    Oled("OLED True Black", isDark = true),
    PaperLight("Paper Light", isDark = false),
}

val NordQuireColors = QuireColors(
    background = Color(0xFF242933),
    sidepane = Color(0xFF2E3440),
    toolbar = Color(0xFF2E3440),
    tabbar = Color(0xFF1E222A),
    paneDivider = Color(0xFF3B4252),
    border = Color(0x33D8DEE9),
    stripe = Color(0x0AD8DEE9),
    accent = Color(0xFF88C0D0),
    accentRed = Color(0xFFBF616A),
    onAccent = Color(0xFF2E3440),
    selectedInactive = Color(0x243B4252),
    fillPrimary = Color(0xFFECEFF4),
    fillSecondary = Color(0xFFD8DEE9),
    fillTertiary = Color(0xFF81A1C1),
    fillQuarternary = Color(0xFF4C566A),
    fillQuinary = Color(0xFF3B4252),
    sectionInfo = Color(0xFF88C0D0),
    sectionAbstract = Color(0xFF81A1C1),
    sectionAttachments = Color(0xFFA3BE8C),
    sectionNotes = Color(0xFFEBCB8B),
    sectionProjects = Color(0xFFB48EAD),
    sectionTags = Color(0xFFD08770),
    sectionRelated = Color(0xFF5E81AC),
    isDark = true,
    surfaceContainerLowest = Color(0xFF1E222A),
    surfaceContainerLow = Color(0xFF242933),
    surfaceContainer = Color(0xFF2E3440),
    surfaceContainerHigh = Color(0xFF3B4252),
    surfaceContainerHighest = Color(0xFF434C5E),
)

val SolarizedQuireColors = QuireColors(
    background = Color(0xFF00212B),
    sidepane = Color(0xFF002B36),
    toolbar = Color(0xFF073642),
    tabbar = Color(0xFF001A22),
    paneDivider = Color(0xFF073642),
    border = Color(0x3393A1A1),
    stripe = Color(0x0A93A1A1),
    accent = Color(0xFF268BD2),
    accentRed = Color(0xFFDC322F),
    onAccent = Color(0xFFFFFFFF),
    selectedInactive = Color(0x2E073642),
    fillPrimary = Color(0xFFEEE8D5),
    fillSecondary = Color(0xFF93A1A1),
    fillTertiary = Color(0xFF657B83),
    fillQuarternary = Color(0xFF586E75),
    fillQuinary = Color(0xFF073642),
    sectionInfo = Color(0xFF268BD2),
    sectionAbstract = Color(0xFF2AA198),
    sectionAttachments = Color(0xFF859900),
    sectionNotes = Color(0xFFB58900),
    sectionProjects = Color(0xFF6C71C4),
    sectionTags = Color(0xFFCB4B16),
    sectionRelated = Color(0xFFD33682),
    isDark = true,
    surfaceContainerLowest = Color(0xFF001820),
    surfaceContainerLow = Color(0xFF00212B),
    surfaceContainer = Color(0xFF002B36),
    surfaceContainerHigh = Color(0xFF073642),
    surfaceContainerHighest = Color(0xFF0E4C5A),
)

val OledQuireColors = QuireColors(
    background = Color(0xFF000000),
    sidepane = Color(0xFF0A0A0A),
    toolbar = Color(0xFF121212),
    tabbar = Color(0xFF050505),
    paneDivider = Color(0xFF1F1F1F),
    border = Color(0x33FFFFFF),
    stripe = Color(0x0DFFFFFF),
    accent = Color(0xFF6366F1),
    accentRed = Color(0xFFF43F5E),
    onAccent = Color(0xFFFFFFFF),
    selectedInactive = Color(0x26FFFFFF),
    fillPrimary = Color(0xFFFFFFFF),
    fillSecondary = Color(0xFFA1A1AA),
    fillTertiary = Color(0xFF71717A),
    fillQuarternary = Color(0xFF3F3F46),
    fillQuinary = Color(0xFF27272A),
    sectionInfo = Color(0xFF818CF8),
    sectionAbstract = Color(0xFF38BDF8),
    sectionAttachments = Color(0xFF34D399),
    sectionNotes = Color(0xFFFBBF24),
    sectionProjects = Color(0xFF22D3EE),
    sectionTags = Color(0xFFFB923C),
    sectionRelated = Color(0xFFC084FC),
    isDark = true,
    surfaceContainerLowest = Color(0xFF000000),
    surfaceContainerLow = Color(0xFF0A0A0A),
    surfaceContainer = Color(0xFF121212),
    surfaceContainerHigh = Color(0xFF1C1C1E),
    surfaceContainerHighest = Color(0xFF2C2C2E),
)

fun quireColors(theme: AppTheme): QuireColors = when (theme) {
    AppTheme.Obsidian -> DarkQuireColors
    AppTheme.Nord -> NordQuireColors
    AppTheme.Solarized -> SolarizedQuireColors
    AppTheme.Oled -> OledQuireColors
    AppTheme.PaperLight -> LightQuireColors
}

enum class ReaderTheme(val title: String) {
    Light("Light"),
    Sepia("Sepia"),
    Solarized("Solarized"),
    Nord("Nord"),
    Dark("Dark"),
    Oled("OLED"),
}

enum class ReaderFont(val title: String) {
    Serif("Serif"),
    Sans("Sans"),
    Mono("Mono"),
}

@Immutable
data class ReaderThemeColors(
    val background: Color,
    val text: Color,
    val surface: Color,
    val border: Color,
)

fun readerColors(theme: ReaderTheme): ReaderThemeColors = when (theme) {
    ReaderTheme.Light -> ReaderThemeColors(
        background = Color(0xFFFFFFFF),
        text = Color(0xFF1E293B),
        surface = Color(0xFFF8FAFC),
        border = Color(0xFFE2E8F0),
    )
    ReaderTheme.Sepia -> ReaderThemeColors(
        background = Color(0xFFFBF0D9),
        text = Color(0xFF433422),
        surface = Color(0xFFF4E6C3),
        border = Color(0xFFE8D6B0),
    )
    ReaderTheme.Solarized -> ReaderThemeColors(
        background = Color(0xFF002B36),
        text = Color(0xFF93A1A1),
        surface = Color(0xFF073642),
        border = Color(0xFF0E4C5A),
    )
    ReaderTheme.Nord -> ReaderThemeColors(
        background = Color(0xFF2E3440),
        text = Color(0xFFECEFF4),
        surface = Color(0xFF3B4252),
        border = Color(0xFF4C566A),
    )
    ReaderTheme.Dark -> ReaderThemeColors(
        background = Color(0xFF18181B),
        text = Color(0xFFF4F4F5),
        surface = Color(0xFF27272A),
        border = Color(0xFF3F3F46),
    )
    ReaderTheme.Oled -> ReaderThemeColors(
        background = Color(0xFF000000),
        text = Color(0xFFE4E4E7),
        surface = Color(0xFF121212),
        border = Color(0xFF27272A),
    )
}
