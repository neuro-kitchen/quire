package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect

/**
 * References dragged from the table onto a project in the projects pane, in window coordinates.
 * Plain Compose (no platform drag and drop): the table starts and moves it, project rows register
 * where they are, and the row under the pointer is the target.
 */
@Stable
class ReferenceDrag {
    /** The references being dragged; empty when nothing is. */
    var ids by mutableStateOf<Set<String>>(emptySet())
        private set
    var position by mutableStateOf(Offset.Zero)
        private set
    /** Where each project row is; state, so the target follows rows that move (a scroll) too. */
    private val targets = mutableStateMapOf<String, Rect>()

    /** The project path under the pointer while dragging. */
    val target: String?
        get() = if (ids.isEmpty()) null else targets.entries.firstOrNull { position in it.value }?.key

    fun start(ids: Set<String>, at: Offset) {
        this.ids = ids
        position = at
    }

    fun move(to: Offset) {
        position = to
    }

    /** Ends the drag: the references and the project they were dropped on, if any. */
    fun end(): Pair<Set<String>, String>? {
        val dropped = target?.let { ids to it }
        ids = emptySet()
        return dropped
    }

    fun cancel() {
        ids = emptySet()
    }

    fun register(path: String, bounds: Rect) {
        // Written on every layout pass: only a change is a new state.
        if (targets[path] != bounds) targets[path] = bounds
    }

    fun unregister(path: String) {
        targets.remove(path)
    }
}
