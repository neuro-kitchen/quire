package dev.neurokitchen.quire.presentation.desktop

import androidx.compose.runtime.Immutable

/** The main window's tabs: the library (always first) and the papers open in the reader. */
@Immutable
data class DesktopState(
    val papers: List<OpenPaper> = emptyList(),
    /** `null`: the library tab. */
    val selected: String? = null,
)

@Immutable
data class OpenPaper(val id: String, val title: String)

sealed interface DesktopEvent {
    /** Opens the paper in a tab (or goes to its tab if already open). */
    data class OnOpenPaper(val id: String, val title: String) : DesktopEvent
    /** `null`: the library tab. */
    data class OnSelectTab(val id: String?) : DesktopEvent
    data class OnCloseTab(val id: String) : DesktopEvent
}
