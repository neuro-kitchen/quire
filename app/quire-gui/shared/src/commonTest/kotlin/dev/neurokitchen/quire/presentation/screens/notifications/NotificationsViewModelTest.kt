package dev.neurokitchen.quire.presentation.screens.notifications

import dev.neurokitchen.quire.domain.model.ActivityItem
import dev.neurokitchen.quire.domain.model.ActivityKind
import dev.neurokitchen.quire.domain.repository.NotificationsRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalCoroutinesApi::class)
class NotificationsViewModelTest {
    @BeforeTest
    fun setUp() = Dispatchers.setMain(UnconfinedTestDispatcher())

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    /** Two unread entries until the list is opened; a sync adding one while open shows at once. */
    private class Fake : NotificationsRepository {
        val items = MutableStateFlow(
            listOf(
                ActivityItem(2, 2, ActivityKind.NewDevice, "Pixel"),
                ActivityItem(1, 1, ActivityKind.Sync, "Laptop", listOf("note" to 2)),
            ),
        )
        val unread = MutableStateFlow(2)
        override fun unread() = unread
        override suspend fun list(limit: Int) = items.value
        override suspend fun markRead() {
            items.value = items.value.map { it.copy(read = true) }
            unread.value = 0
        }
    }

    @Test
    fun theBadgeCountsUnreadAndOpeningReadsThem() = runTest {
        val fake = Fake()
        val vm = NotificationsViewModel(fake)
        assertEquals(2, vm.state.first { it.unread == 2 }.unread)
        vm.onEvent(NotificationsEvent.OnOpen)
        val open = vm.state.first { it.items != null && it.unread == 0 }
        assertEquals(listOf(ActivityKind.NewDevice, ActivityKind.Sync), open.items!!.map { it.kind })

        // A sync while the list is open: the new entry shows, and is read.
        fake.items.value = listOf(ActivityItem(3, 3, ActivityKind.KeyChanged, null)) + fake.items.value
        fake.unread.value = 1
        assertEquals(3, vm.state.first { it.items!!.size == 3 }.items!!.size)
        assertEquals(0, vm.state.first { it.unread == 0 }.unread)

        vm.onEvent(NotificationsEvent.OnClose)
        fake.unread.value = 4
        assertEquals(4, vm.state.first { it.unread == 4 }.unread, "closed: the badge counts again")
    }
}
