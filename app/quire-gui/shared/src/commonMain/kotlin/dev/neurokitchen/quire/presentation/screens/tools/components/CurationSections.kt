package dev.neurokitchen.quire.presentation.screens.tools.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.tools.ToolsConfirm
import dev.neurokitchen.quire.presentation.screens.tools.ToolsEvent
import dev.neurokitchen.quire.presentation.screens.tools.ToolsState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.tools_check
import dev.neurokitchen.quire.resources.tools_check_retractions
import dev.neurokitchen.quire.resources.tools_checks_hint
import dev.neurokitchen.quire.resources.tools_duplicates_hint
import dev.neurokitchen.quire.resources.tools_find_duplicates
import dev.neurokitchen.quire.resources.tools_merge
import dev.neurokitchen.quire.resources.tools_no_duplicates
import dev.neurokitchen.quire.resources.tools_no_issues
import dev.neurokitchen.quire.resources.tools_no_retractions
import dev.neurokitchen.quire.resources.tools_not_duplicate
import dev.neurokitchen.quire.resources.tools_retractions_hint
import dev.neurokitchen.quire.resources.tools_section_checks
import dev.neurokitchen.quire.resources.tools_section_duplicates
import dev.neurokitchen.quire.resources.tools_shared_identifier
import dev.neurokitchen.quire.resources.tools_shared_title
import org.jetbrains.compose.resources.stringResource

/** Groups of references that look like one work: merge one into the first, or mark it distinct. */
@Composable
internal fun DuplicatesSection(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    SectionHeader(Res.string.tools_section_duplicates, Res.string.tools_duplicates_hint)
    SecondaryAction(Res.string.tools_find_duplicates, enabled = !state.isWorking) { onEvent(ToolsEvent.OnFindDuplicates) }
    Spacer(Modifier.height(Dimens.SpaceSm))
    val groups = state.duplicates ?: return
    if (groups.isEmpty()) Hint(stringResource(Res.string.tools_no_duplicates))
    groups.forEach { group ->
        Group {
            ReferenceText(group.keep)
            group.others.forEach { other ->
                HorizontalDivider(color = QuireTheme.colors.fillQuarternary, modifier = Modifier.padding(vertical = Dimens.SpaceXs))
                ReferenceText(other.reference)
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                    Hint(other.sharedIdentifier?.let { stringResource(Res.string.tools_shared_identifier, it) } ?: stringResource(Res.string.tools_shared_title))
                    SmallAction(Res.string.tools_merge, enabled = !state.isWorking) {
                        onEvent(ToolsEvent.OnConfirm(ToolsConfirm.Merge(keep = group.keep, other = other.reference)))
                    }
                    SmallAction(Res.string.tools_not_duplicate, enabled = !state.isWorking) {
                        onEvent(ToolsEvent.OnDismissDuplicate(group.keep.id, other.reference.id))
                    }
                }
            }
        }
    }
}

/** The library's issues, and retraction notices (online, on request). */
@Composable
internal fun ChecksSection(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    SectionHeader(Res.string.tools_section_checks, Res.string.tools_checks_hint)
    PrimaryAction(Res.string.tools_check, enabled = !state.isWorking) { onEvent(ToolsEvent.OnCheck) }
    Spacer(Modifier.height(Dimens.SpaceSm))
    state.checks?.let { checks ->
        if (checks.isEmpty()) Hint(stringResource(Res.string.tools_no_issues))
        checks.forEach { checked ->
            Group {
                Text(checked.citeKey, style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillPrimary)
                checked.issues.forEach { Hint("• $it") }
            }
        }
    }
    Spacer(Modifier.height(Dimens.SpaceMd))
    Hint(stringResource(Res.string.tools_retractions_hint))
    Spacer(Modifier.height(Dimens.SpaceXs))
    SecondaryAction(Res.string.tools_check_retractions, enabled = !state.isWorking) { onEvent(ToolsEvent.OnCheckRetractions) }
    Spacer(Modifier.height(Dimens.SpaceSm))
    state.retractions?.let { results ->
        if (results.isEmpty()) Hint(stringResource(Res.string.tools_no_retractions))
        results.forEach { result ->
            Group {
                ReferenceText(result.reference)
                result.notices.forEach { Text("• $it", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error) }
                result.error?.let { Hint(it) }
            }
        }
    }
}
