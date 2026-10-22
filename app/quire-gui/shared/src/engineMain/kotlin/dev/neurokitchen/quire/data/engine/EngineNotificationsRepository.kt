package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.ActivityItem
import dev.neurokitchen.quire.domain.model.ActivityKind
import dev.neurokitchen.quire.domain.repository.NotificationsRepository
import dev.neurokitchen.quire.ffi.ActivityRecord

/** This device's notifications through the engine (`quire-ffi` `activity`). */
class EngineNotificationsRepository(private val session: EngineSession) : NotificationsRepository {
    override fun unread() = session.reloading { it.unreadActivity().toInt() }

    override suspend fun list(limit: Int) = session.library().activity(limit.toUInt()).map { it.toDomain() }

    override suspend fun markRead() = session.library().markActivityRead()
}

internal fun ActivityRecord.toDomain(): ActivityItem {
    val kind = when (kind) {
        "sync" -> ActivityKind.Sync
        "linked" -> ActivityKind.Linked
        "link_allowed" -> ActivityKind.LinkAllowed
        "link_refused" -> ActivityKind.LinkRefused
        "wrong_code" -> ActivityKind.WrongCode
        "unlinked" -> ActivityKind.Unlinked
        "key_changed" -> ActivityKind.KeyChanged
        "new_device" -> ActivityKind.NewDevice
        else -> ActivityKind.Other
    }
    // `sync`: `kind=count,…`; others: a number, or nothing.
    val counts = if (kind == ActivityKind.Sync) {
        detail.split(',').mapNotNull { part ->
            val (name, count) = part.split('=').takeIf { it.size == 2 } ?: return@mapNotNull null
            count.toIntOrNull()?.let { name to it }
        }
    } else {
        emptyList()
    }
    return ActivityItem(id, at, kind, source, counts, detail.toIntOrNull(), read)
}
