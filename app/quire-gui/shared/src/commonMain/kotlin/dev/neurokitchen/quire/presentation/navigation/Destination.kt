package dev.neurokitchen.quire.presentation.navigation

import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.destination_home
import dev.neurokitchen.quire.resources.destination_library
import dev.neurokitchen.quire.resources.destination_search
import dev.neurokitchen.quire.resources.destination_settings
import dev.neurokitchen.quire.resources.ic_home
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.ic_settings
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.StringResource

/** Top-level places on phones and tablets (the desktop app uses its menu bar instead). */
enum class Destination(val label: StringResource, val icon: DrawableResource) {
    Home(Res.string.destination_home, Res.drawable.ic_home),
    Library(Res.string.destination_library, Res.drawable.ic_library_books),
    Search(Res.string.destination_search, Res.drawable.ic_search),
    Settings(Res.string.destination_settings, Res.drawable.ic_settings),
}
