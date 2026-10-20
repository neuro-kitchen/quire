package dev.neurokitchen.quire.domain.model

/** A tag with how many references carry it; [color] `#rrggbb` for coloured tags. */
data class Tag(val name: String, val color: String? = null, val count: Int = 0)
