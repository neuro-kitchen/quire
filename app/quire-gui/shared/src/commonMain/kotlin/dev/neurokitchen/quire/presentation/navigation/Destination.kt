package dev.neurokitchen.quire.presentation.navigation

import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.destination_library
import dev.neurokitchen.quire.resources.destination_settings
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_settings
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.StringResource

/** Top-level places on phones and tablets (the desktop app uses its menu bar instead). */
enum class Destination(val label: StringResource, val icon: DrawableResource) {
    Library(Res.string.destination_library, Res.drawable.ic_library_books),
    Settings(Res.string.destination_settings, Res.drawable.ic_settings),
}
