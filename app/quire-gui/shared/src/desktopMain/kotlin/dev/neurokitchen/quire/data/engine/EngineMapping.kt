package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.BlockKind
import dev.neurokitchen.quire.domain.model.Creator
import dev.neurokitchen.quire.domain.model.Paper
import dev.neurokitchen.quire.domain.model.PaperTab
import dev.neurokitchen.quire.domain.model.PaperTabKind
import dev.neurokitchen.quire.domain.model.TextBlock
import dev.neurokitchen.quire.domain.model.Word
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.Project
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.model.SortColumn
import dev.neurokitchen.quire.ffi.BlockRecord
import dev.neurokitchen.quire.ffi.PaperRecord
import dev.neurokitchen.quire.ffi.ProjectRecord
import dev.neurokitchen.quire.ffi.Query
import dev.neurokitchen.quire.ffi.ReferenceRecord
import dev.neurokitchen.quire.ffi.Source
import dev.neurokitchen.quire.ffi.BlockKind as FfiBlockKind
import dev.neurokitchen.quire.ffi.SortColumn as FfiSortColumn

// Engine records ↔ the app's domain types. Plain field copies: no rules here.

internal fun LibraryQuery.toFfi() = Query(
    source = when (val source = source) {
        LibrarySource.All -> Source.All
        LibrarySource.Unfiled -> Source.Unfiled
        LibrarySource.Trash -> Source.Trash
        is LibrarySource.InProject -> Source.Project(source.path)
    },
    text = text,
    tags = tags.toList(),
    sort = when (sort) {
        SortColumn.Title -> FfiSortColumn.TITLE
        SortColumn.Creator -> FfiSortColumn.CREATOR
        SortColumn.Year -> FfiSortColumn.YEAR
        SortColumn.DateAdded -> FfiSortColumn.DATE_ADDED
    },
    ascending = ascending,
)

internal fun ReferenceRecord.toDomain() = Reference(
    id = id,
    citeKey = citeKey,
    kind = kind,
    title = title,
    creators = creators.map { Creator(sortName = it.sortName, displayName = it.displayName, role = it.role) },
    year = year?.toInt(),
    date = date.orEmpty(),
    containerTitle = containerTitle.orEmpty(),
    publisher = publisher.orEmpty(),
    volume = volume.orEmpty(),
    issue = issue.orEmpty(),
    pages = pages.orEmpty(),
    doi = doi.orEmpty(),
    url = url.orEmpty(),
    abstract = abstractText.orEmpty(),
    tags = tags,
    projects = projects,
    hasPdf = hasPdf,
    reading = readingStatus(reading),
    dateAdded = dateAdded,
)

internal fun ProjectRecord.toDomain() = Project(
    path = path,
    name = name,
    parentPath = parent,
    referenceCount = references.toInt(),
)

internal fun readingStatus(text: String) = when (text) {
    "reading" -> ReadingStatus.Reading
    "read" -> ReadingStatus.Read
    else -> ReadingStatus.Unread
}

internal fun PaperRecord.toDomain() = Paper(
    tabs = tabs.mapNotNull { tab ->
        val kind = when (tab.name) {
            "abstract" -> PaperTabKind.Abstract
            "paper" -> PaperTabKind.Paper
            "front" -> PaperTabKind.Front
            "figures" -> PaperTabKind.Figures
            "tables" -> PaperTabKind.Tables
            "equations" -> PaperTabKind.Equations
            "references" -> PaperTabKind.References
            else -> return@mapNotNull null
        }
        PaperTab(kind, tab.blocks.map { it.toDomain() }, tab.count?.toInt())
    },
    source = source,
    textError = textError,
)

private fun BlockRecord.toDomain() = TextBlock(
    kind = when (val k = kind) {
        is FfiBlockKind.Heading -> BlockKind.Heading(k.level.toInt())
        FfiBlockKind.Paragraph -> BlockKind.Paragraph
        FfiBlockKind.ListItem -> BlockKind.ListItem
        FfiBlockKind.TableRow -> BlockKind.TableRow
        FfiBlockKind.Equation -> BlockKind.Equation
        is FfiBlockKind.Image -> BlockKind.Image(k.url)
    },
    words = words.map { Word(it.text, it.bold, it.italic, it.mention) },
)
