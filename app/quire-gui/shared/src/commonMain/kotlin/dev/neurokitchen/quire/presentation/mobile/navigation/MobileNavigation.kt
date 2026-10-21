package dev.neurokitchen.quire.presentation.mobile.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSerializable
import androidx.compose.runtime.setValue
import androidx.lifecycle.viewmodel.navigation3.rememberViewModelStoreNavEntryDecorator
import androidx.navigation3.runtime.NavBackStack
import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.rememberDecoratedNavEntries
import androidx.navigation3.runtime.rememberNavBackStack
import androidx.navigation3.runtime.rememberSaveableStateHolderNavEntryDecorator
import androidx.savedstate.compose.serialization.serializers.MutableStateSerializer

/**
 * The phone app's navigation: a back stack per tab (Navigation 3's "multiple back stacks" recipe).
 * Back pops the current tab's stack; at a tab's root it returns to Home, and at Home's root it
 * leaves the app ("exit through Home"). Every entry has its own ViewModels and saved state, kept
 * while it is in a stack, also when its tab is not shown.
 */
class MobileNavigation internal constructor(
    tab: MutableState<Tab>,
    private val stacks: Map<Tab, NavBackStack<NavKey>>,
) {
    var tab: Tab by tab
        private set

    /** What is on screen. */
    val current: NavKey get() = stack(tab).last()

    /** On a tab's root: the bottom bar is shown. */
    val isAtTabRoot: Boolean get() = stack(tab).size == 1

    /** A tab tapped: shown; tapped again while shown, back to its root. */
    fun select(target: Tab) {
        if (target == tab) {
            val stack = stack(target)
            while (stack.size > 1) stack.removeAt(stack.lastIndex)
        } else {
            tab = target
        }
    }

    /** Opens [route] on top of the current tab. */
    fun open(route: (Tab) -> Route) {
        stack(tab).add(route(tab))
    }

    fun back() {
        val stack = stack(tab)
        when {
            stack.size > 1 -> stack.removeAt(stack.lastIndex)
            tab != Tab.Home -> tab = Tab.Home
        }
    }

    /** The keys shown, oldest first: Home's stack, then the current tab's (so Back reaches Home). */
    internal fun visibleTabs(): List<Tab> = if (tab == Tab.Home) listOf(Tab.Home) else listOf(Tab.Home, tab)

    internal fun stack(tab: Tab): NavBackStack<NavKey> = stacks.getValue(tab)
}

@Composable
fun rememberMobileNavigation(): MobileNavigation {
    val tab = rememberSerializable(serializer = MutableStateSerializer(Tab.serializer())) { mutableStateOf(Tab.Home) }
    val stacks = Tab.entries.associateWith { rememberNavBackStack(routeSavedState, it.root) }
    return remember { MobileNavigation(tab, stacks) }
}

/**
 * The entries to show: each tab's stack decorated with its own saved state and ViewModel store
 * (so leaving a tab keeps its state), then Home's and the current tab's, in that order.
 */
@Composable
fun MobileNavigation.entries(entryProvider: (NavKey) -> NavEntry<NavKey>): List<NavEntry<NavKey>> {
    val decorated = Tab.entries.associateWith { tab ->
        rememberDecoratedNavEntries(
            backStack = stack(tab),
            entryDecorators = listOf(
                rememberSaveableStateHolderNavEntryDecorator(),
                rememberViewModelStoreNavEntryDecorator(),
            ),
            entryProvider = entryProvider,
        )
    }
    return visibleTabs().flatMap { decorated.getValue(it) }
}
