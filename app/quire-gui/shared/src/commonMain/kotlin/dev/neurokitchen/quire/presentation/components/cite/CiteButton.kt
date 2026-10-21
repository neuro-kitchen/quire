package dev.neurokitchen.quire.presentation.components.cite

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.cite_biblatex
import dev.neurokitchen.quire.resources.cite_bibtex
import dev.neurokitchen.quire.resources.cite_copy
import dev.neurokitchen.quire.resources.cite_csl_json
import dev.neurokitchen.quire.resources.cite_key
import dev.neurokitchen.quire.resources.ic_format_quote
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/**
 * Cite: a quote button whose menu copies the reference as a cite key or an entry (BibLaTeX,
 * BibTeX, CSL-JSON). The menu stays open while the entry is written, and closes once it is on the
 * clipboard (Android confirms the copy itself); it shows why when it could not be written.
 */
@Composable
fun CiteButton(id: String, citeKey: String, modifier: Modifier = Modifier, viewModel: CiteViewModel = koinViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val clipboard = LocalClipboardManager.current
    val colors = QuireTheme.colors
    var open by remember { mutableStateOf(false) }
    // The menu this card opened (one ViewModel serves every card of the screen).
    var asked by remember { mutableStateOf(false) }
    LaunchedEffect(state.copied, asked) {
        val text = state.copied
        if (text != null && asked) {
            clipboard.setText(AnnotatedString(text))
            viewModel.onEvent(CiteEvent.OnCopied)
            asked = false
            open = false
        }
    }
    Box(modifier) {
        val label = stringResource(Res.string.cite_copy)
        IconButton(onClick = { open = true }, modifier = Modifier.size(36.dp)) {
            Icon(painterResource(Res.drawable.ic_format_quote), contentDescription = label, tint = colors.fillSecondary, modifier = Modifier.size(20.dp))
        }
        DropdownMenu(
            expanded = open,
            onDismissRequest = {
                open = false
                asked = false
                viewModel.onEvent(CiteEvent.OnDismissError)
            },
        ) {
            Text(
                label,
                style = MaterialTheme.typography.labelMedium,
                color = colors.fillSecondary,
                modifier = Modifier.padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceXs),
            )
            CiteFormat.entries.forEach { format ->
                DropdownMenuItem(
                    text = { Text(stringResource(format.label)) },
                    onClick = {
                        asked = true
                        viewModel.onEvent(CiteEvent.OnCopy(id, citeKey, format))
                    },
                    enabled = state.working == null,
                    trailingIcon = if (state.working == format && asked) {
                        { CircularProgressIndicator(Modifier.size(16.dp), strokeWidth = 2.dp) }
                    } else {
                        null
                    },
                )
            }
            state.error?.takeIf { asked }?.let { error ->
                Text(
                    error,
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.accentRed,
                    modifier = Modifier.widthIn(max = 260.dp).padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceXs),
                )
            }
        }
    }
}

private val CiteFormat.label: StringResource
    get() = when (this) {
        CiteFormat.Key -> Res.string.cite_key
        CiteFormat.Biblatex -> Res.string.cite_biblatex
        CiteFormat.Bibtex -> Res.string.cite_bibtex
        CiteFormat.CslJson -> Res.string.cite_csl_json
    }
