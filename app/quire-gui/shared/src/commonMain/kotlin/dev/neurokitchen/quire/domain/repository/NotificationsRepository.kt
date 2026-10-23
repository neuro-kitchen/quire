package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.domain.model.ActivityItem
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOf

/** This device's notifications: what syncs brought from which device, and security events. */
interface NotificationsRepository {
    /** How many are unread, again after each change (a sync adds some). */
    fun unread(): Flow<Int> = flowOf(0)
    /** The newest first. */
    suspend fun list(limit: Int = 100): List<ActivityItem> = emptyList()
    suspend fun markRead() {}
}
