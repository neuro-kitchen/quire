package dev.neurokitchen.quire.domain.model

/** What a notification is about (`.tasks/T14-security.md`). */
enum class ActivityKind {
    /** A sync brought changes from one of the person's devices ([ActivityItem.counts]). */
    Sync,
    /** This device was linked to the person's devices. */
    Linked,
    /** This device let a new one in ([ActivityItem.source]). */
    LinkAllowed,
    LinkRefused,
    /** Wrong link codes were tried ([ActivityItem.number]). */
    WrongCode,
    /** This device unlinked one; the library key changed. */
    Unlinked,
    /** Another device unlinked one; the library key changed. */
    KeyChanged,
    /** A device this one had not seen before is on the hub. */
    NewDevice,
    Other,
    ;

    /** Worth the person's attention (a possible intruder, or a recovery kit out of date). */
    val isWarning: Boolean get() = this == WrongCode || this == NewDevice || this == Unlinked || this == KeyChanged
}

/**
 * One notification, kept on this device only: [at] in milliseconds since 1970; [source] the
 * device it concerns, by name; [counts] what a sync brought, by kind (reference, note, highlight,
 * reading, project, tag, attachment, …); [number] a count or key epoch.
 */
data class ActivityItem(
    val id: Long,
    val at: Long,
    val kind: ActivityKind,
    val source: String?,
    val counts: List<Pair<String, Int>> = emptyList(),
    val number: Int? = null,
    val read: Boolean = false,
)
