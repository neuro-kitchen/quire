package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.domain.model.AddOutcome
import dev.neurokitchen.quire.domain.model.FieldChange
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.LookupFailure
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.domain.model.Project
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.model.ReferenceIssue
import dev.neurokitchen.quire.domain.model.Tag
import kotlinx.coroutines.flow.Flow

/**
 * The library, as the screens need it. Rules (search, sorting, merging, validation) belong to the
 * Rust engine; implementations only translate its calls and change stream. Flows emit again after
 * every change to the library and fail with the engine's message.
 */
interface LibraryRepository {
    /** The open library's name and folder, for titles. */
    suspend fun describe(): LibraryDescription

    /** References answering `query`. */
    fun references(query: LibraryQuery): Flow<List<Reference>>

    /** Every project, parents before their children. */
    fun projects(): Flow<List<Project>>

    /** Opens the library again, e.g. after its settings changed; open flows switch to it. */
    suspend fun reopen()

    /** Adds a reference typed in by hand (its fields; `id` and `citeKey` are the engine's). */
    suspend fun addReference(reference: Reference): AddOutcome = unsupported()
    /** Replaces the fields of the reference with `reference.id`; changed fields become the user's. */
    suspend fun updateReference(reference: Reference): Unit = Unit
    /** Moves a reference to the trash ([restoreReference] takes it back). */
    suspend fun deleteReference(id: String): Unit = Unit
    /** Creates a project; returns its path. */
    suspend fun createProject(name: String, parentPath: String? = null): String = unsupported()
    /** Moves the project and its subprojects to the trash; their references stay in the library. */
    suspend fun deleteProject(path: String): Unit = Unit

    /** What `input` (DOI, arXiv ID, PubMed ID, URL) names, looked up but not added: a preview. */
    suspend fun lookup(input: String): Reference = unsupported()
    /** Adds what `input` names, into the project at `projectPath` if given. */
    suspend fun addByIdentifier(input: String, projectPath: String? = null): AddOutcome = unsupported()
    /** Adds the work the page at `url` describes, from its `html` as the browser loaded it. */
    suspend fun addFromPage(url: String, html: String, projectPath: String? = null): AddOutcome = unsupported()
    suspend fun restoreReference(id: String): Unit = unsupported()
    /** Looks a reference up again at its source; fields the user edited are kept. */
    suspend fun refetch(id: String, dryRun: Boolean = false): List<FieldChange> = unsupported()
    /** What changing `reference`'s fields would change, without writing. */
    suspend fun previewUpdate(reference: Reference): List<FieldChange> = unsupported()
    suspend fun issues(id: String): List<ReferenceIssue> = unsupported()
    /** Inputs whose lookup failed. */
    suspend fun failures(): List<LookupFailure> = unsupported()
    /** `false` if `input` was not a recorded failure. */
    suspend fun forgetFailure(input: String): Boolean = unsupported()

    /** Every tag with its count, coloured ones first; again after every change. */
    fun tags(): Flow<List<Tag>> = unsupported()
    suspend fun tag(id: String, names: List<String>): Unit = unsupported()
    suspend fun untag(id: String, names: List<String>): Unit = unsupported()
    /** `true` if it merged into an existing tag of that name. */
    suspend fun renameTag(old: String, new: String): Boolean = unsupported()
    /** Returns how many references had it. */
    suspend fun deleteTag(name: String): Int = unsupported()
    /** `color` `#rrggbb` (`null` removes it), at `position` 1–9 or the first free one. */
    suspend fun colorTag(name: String, color: String?, position: Int? = null): Tag = unsupported()

    /** Returns the project's new path. */
    suspend fun renameProject(path: String, name: String): String = unsupported()
    /** `false` if it was already in the project. */
    suspend fun addToProject(id: String, projectPath: String): Boolean = unsupported()
    /** `false` if it was not in the project; the reference stays in the library. */
    suspend fun removeFromProject(id: String, projectPath: String): Boolean = unsupported()

    /** References related to a reference (both ways). */
    suspend fun related(id: String): List<Reference> = unsupported()
    /** `false` if they already were. */
    suspend fun relate(a: String, b: String): Boolean = unsupported()
    suspend fun unrelate(a: String, b: String): Boolean = unsupported()

    /** Notes on no reference; again after every change. */
    fun standaloneNotes(): Flow<List<Note>> = unsupported()
    suspend fun addStandaloneNote(body: String): Unit = unsupported()
}

/** For implementations without an engine (samples, previews). */
internal fun unsupported(): Nothing = throw UnsupportedOperationException("needs the engine")

data class LibraryDescription(val name: String, val path: String, val configFile: String)
