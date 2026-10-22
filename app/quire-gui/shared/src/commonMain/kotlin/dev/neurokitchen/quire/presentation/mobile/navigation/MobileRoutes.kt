package dev.neurokitchen.quire.presentation.mobile.navigation

import androidx.navigation3.runtime.NavKey
import androidx.savedstate.serialization.SavedStateConfiguration
import dev.neurokitchen.quire.domain.model.LibrarySource
import kotlinx.serialization.Serializable
import kotlinx.serialization.modules.SerializersModule
import kotlinx.serialization.modules.polymorphic
import kotlinx.serialization.modules.subclassesOfSealed

/** The four tabs of the phone app; each keeps its own back stack. */
@Serializable
enum class Tab { Home, Library, Search, Settings }

/**
 * Every place in the phone app. Tab roots are the bottom bar's tabs; the others are opened on top of
 * the tab they were opened from ([tab]), so the same paper can be open in two tabs at once.
 */
@Serializable
sealed interface Route : NavKey {
    @Serializable data object Home : Route
    @Serializable data object Library : Route
    @Serializable data object Search : Route
    @Serializable data object Settings : Route

    /** A list of references: the library, unfiled, the trash, or a project. */
    @Serializable data class Shelf(val tab: Tab, val shelf: ShelfKey) : Route

    /** A reference's details. */
    @Serializable data class Details(val tab: Tab, val id: String) : Route

    /** A paper in the reader. */
    @Serializable data class Reader(val tab: Tab, val id: String, val title: String) : Route

    /** A page of the settings. */
    @Serializable data class SettingsPage(val tab: Tab, val page: MobileSettingsPage) : Route

    /** Adding references (every way in). */
    @Serializable data class Add(
        val tab: Tab,
        val projectPath: String? = null,
        /** Shared identifiers (one per line), to add. */
        val identifiers: String = "",
        /** Shared PDF files (paths in the cache), to add. */
        val pdfs: List<String> = emptyList(),
        /** The way in to show first (an `AddTab` name), from the Add menu. */
        val way: String? = null,
    ) : Route
}

/**
 * What a [Route.Shelf] lists, in a form that survives process death: a source ([path] for a
 * project), or the library narrowed to a tag or a saved search ([name]).
 */
@Serializable
data class ShelfKey(val kind: Kind, val path: String? = null, val name: String = "") {
    enum class Kind { All, Unfiled, Trash, Project, Tag, SavedSearch }

    fun toSource(): LibrarySource = when (kind) {
        Kind.Unfiled -> LibrarySource.Unfiled
        Kind.Trash -> LibrarySource.Trash
        Kind.Project -> LibrarySource.InProject(path.orEmpty())
        Kind.All, Kind.Tag, Kind.SavedSearch -> LibrarySource.All
    }
}

val Tab.root: Route
    get() = when (this) {
        Tab.Home -> Route.Home
        Tab.Library -> Route.Library
        Tab.Search -> Route.Search
        Tab.Settings -> Route.Settings
    }

/** Saving the back stacks on every platform (iOS has no reflection): the routes, listed. */
internal val routeSavedState = SavedStateConfiguration {
    serializersModule = SerializersModule {
        polymorphic(NavKey::class) { subclassesOfSealed<Route>() }
    }
}

/** The phone's settings pages (the desktop's shortcuts and config file editor are not among them). */
@Serializable
enum class MobileSettingsPage { Appearance, Reading, Account, Libraries, Sync, About }
