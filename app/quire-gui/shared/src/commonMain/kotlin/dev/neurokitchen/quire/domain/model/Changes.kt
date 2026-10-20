package dev.neurokitchen.quire.domain.model

/** What adding did; [isNew] `false` when the work was already in the library (then only linked). */
data class AddOutcome(val isNew: Boolean, val reference: Reference)

/** A field an edit or a refetch changed (or would change); [kept] when the user's value stays. */
data class FieldChange(val field: String, val old: String?, val new: String?, val kept: Boolean)

/** A problem the engine found with a reference (a missing field, a failed lookup, a retraction…). */
data class ReferenceIssue(val code: String, val message: String)

/** An input whose lookup failed, to retry or forget. */
data class LookupFailure(val input: String, val error: String, val attempts: Int, val projectPath: String?)
