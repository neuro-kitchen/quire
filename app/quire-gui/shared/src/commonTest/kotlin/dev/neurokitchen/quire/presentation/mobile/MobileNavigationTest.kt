package dev.neurokitchen.quire.presentation.mobile

import androidx.compose.runtime.mutableStateOf
import androidx.navigation3.runtime.NavBackStack
import androidx.navigation3.runtime.NavKey
import dev.neurokitchen.quire.presentation.mobile.navigation.MobileNavigation
import dev.neurokitchen.quire.presentation.mobile.navigation.Route
import dev.neurokitchen.quire.presentation.mobile.navigation.ShelfKey
import dev.neurokitchen.quire.presentation.mobile.navigation.Tab
import dev.neurokitchen.quire.presentation.mobile.navigation.root
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class MobileNavigationTest {
    private fun navigation() = MobileNavigation(
        mutableStateOf(Tab.Home),
        Tab.entries.associateWith { NavBackStack<NavKey>(it.root) },
    )

    /** Back used to leave the app from anywhere: there was no back stack. */
    @Test
    fun backPopsTheTabThenReturnsHome() {
        val nav = navigation()
        nav.select(Tab.Library)
        nav.open { Route.Shelf(it, ShelfKey(ShelfKey.Kind.Project, "thesis", "Thesis")) }
        nav.open { Route.Details(it, "3") }
        assertEquals(Route.Details(Tab.Library, "3"), nav.current)
        assertFalse(nav.isAtTabRoot, "the bottom bar hides on pushed screens")
        nav.back()
        assertEquals(Route.Shelf(Tab.Library, ShelfKey(ShelfKey.Kind.Project, "thesis", "Thesis")), nav.current)
        nav.back()
        assertEquals(Route.Library, nav.current)
        nav.back()
        assertEquals(Tab.Home to Route.Home, nav.tab to nav.current, "exit through Home")
    }

    @Test
    fun eachTabKeepsItsStackAndReselectingReturnsToItsRoot() {
        val nav = navigation()
        nav.select(Tab.Search)
        nav.open { Route.Details(it, "1") }
        nav.select(Tab.Library)
        assertEquals(Route.Library, nav.current)
        nav.select(Tab.Search)
        assertEquals(Route.Details(Tab.Search, "1"), nav.current, "Search kept its stack")
        nav.select(Tab.Search)
        assertEquals(Route.Search, nav.current)
        assertTrue(nav.isAtTabRoot)
    }

    @Test
    fun swipeMovesBetweenTabsOnlyAtTheirRoot() {
        val nav = navigation()
        nav.swipe(forward = false)
        assertEquals(Tab.Home, nav.tab, "nothing before Home")
        nav.swipe(forward = true)
        assertEquals(Tab.Library, nav.tab)
        nav.open { Route.Details(it, "2") }
        nav.swipe(forward = true)
        assertEquals(Tab.Library, nav.tab, "no swipe on a pushed screen")
        nav.back()
        nav.swipe(forward = true)
        nav.swipe(forward = true)
        nav.swipe(forward = true)
        assertEquals(Tab.Settings, nav.tab, "nothing after Settings")
        nav.swipe(forward = false)
        assertEquals(Tab.Search, nav.tab)
    }
}
