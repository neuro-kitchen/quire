package dev.neurokitchen.quire.presentation.designsystem.components

import kotlin.test.Test
import kotlin.test.assertEquals

class ResizableRowTest {
    private fun assertNear(expected: List<Float>, actual: List<Float>) {
        assertEquals(expected.size, actual.size)
        expected.zip(actual).forEach { (e, a) -> assertEquals(e, a, 0.0001f, "$expected vs $actual") }
    }

    /**
     * The old split turned a drag into a ratio with a fixed 800 px: at any other width the
     * boundary did not follow the mouse. A drag is a fraction of the row's real width.
     */
    @Test
    fun aDragMovesTheBoundaryByItsShareOfTheWidth() {
        // 100 dp to the right in a 1000 dp row: a tenth.
        assertNear(listOf(0.6f, 0.4f), resized(listOf(0.5f, 0.5f), boundary = 0, delta = 100f, width = 1000f, min = 0.15f))
        assertNear(listOf(0.55f, 0.45f), resized(listOf(0.5f, 0.5f), boundary = 0, delta = 100f, width = 2000f, min = 0.15f))
    }

    @Test
    fun onlyTheTwoColumnsBesideTheBoundaryChange() {
        assertNear(listOf(0.4f, 0.25f, 0.35f), resized(listOf(0.4f, 0.35f, 0.25f), boundary = 1, delta = -100f, width = 1000f, min = 0.15f))
    }

    @Test
    fun noColumnGetsNarrowerThanTheLeast() {
        assertNear(listOf(0.85f, 0.15f), resized(listOf(0.5f, 0.5f), boundary = 0, delta = 900f, width = 1000f, min = 0.15f))
        assertNear(listOf(0.15f, 0.85f), resized(listOf(0.5f, 0.5f), boundary = 0, delta = -900f, width = 1000f, min = 0.15f))
    }

    @Test
    fun savedFractionsThatDoNotFitAreEvenedOut() {
        assertNear(listOf(0.5f, 0.5f), fitted(listOf(0.2f, 0.3f, 0.5f), count = 2))
        assertNear(listOf(0.3f, 0.7f), fitted(listOf(0.3f, 0.7f), count = 2))
        assertNear(listOf(1 / 3f, 1 / 3f, 1 / 3f), fitted(listOf(0.9f, 0.05f, 0.05f), count = 3, min = 0.15f))
    }
}
