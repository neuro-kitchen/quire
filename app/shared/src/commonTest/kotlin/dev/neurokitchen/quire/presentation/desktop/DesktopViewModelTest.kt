package dev.neurokitchen.quire.presentation.desktop

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class DesktopViewModelTest {
    @Test
    fun papersOpenOnceAndClosingMovesLeft() {
        val vm = DesktopViewModel()
        vm.onEvent(DesktopEvent.OnOpenPaper("a", "A"))
        vm.onEvent(DesktopEvent.OnOpenPaper("b", "B"))
        vm.onEvent(DesktopEvent.OnOpenPaper("a", "A"))
        assertEquals(listOf("a", "b"), vm.state.value.papers.map { it.id }, "no second tab for a")
        assertEquals("a", vm.state.value.selected)

        vm.onEvent(DesktopEvent.OnSelectTab("b"))
        vm.onEvent(DesktopEvent.OnCloseTab("b"))
        assertEquals("a", vm.state.value.selected, "its left neighbour")
        vm.onEvent(DesktopEvent.OnCloseTab("a"))
        assertNull(vm.state.value.selected, "back to the library")
    }
}
