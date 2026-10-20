package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.PrimaryTabRow
import androidx.compose.material3.Tab
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import dev.neurokitchen.quire.domain.model.Creator
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_add
import dev.neurokitchen.quire.resources.ic_auto_fix_high
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import org.jetbrains.compose.resources.painterResource

@Composable
fun AddReferenceDialog(
    onDismiss: () -> Unit,
    onAdd: (Reference) -> Unit,
) {
    var selectedTab by remember { mutableIntStateOf(0) }
    val scope = rememberCoroutineScope()

    // Identifier lookup state
    var identifierInput by remember { mutableStateOf("10.1038/s41586-021-03819-2") }
    var isLookingUp by remember { mutableStateOf(false) }
    var lookupPreview by remember { mutableStateOf<Reference?>(null) }

    // Manual form state
    var title by remember { mutableStateOf("") }
    var authors by remember { mutableStateOf("") }
    var year by remember { mutableStateOf("2026") }
    var containerTitle by remember { mutableStateOf("") }
    var doi by remember { mutableStateOf("") }
    var tagsInput by remember { mutableStateOf("") }

    // BibTeX state
    var bibtexInput by remember { mutableStateOf("") }

    Dialog(onDismissRequest = onDismiss) {
        Card(
            shape = RoundedCornerShape(Dimens.RadiusLarge),
            colors = CardDefaults.cardColors(
                containerColor = MaterialTheme.colorScheme.surfaceContainer,
            ),
            elevation = CardDefaults.cardElevation(defaultElevation = Dimens.ElevationMedium),
            modifier = Modifier.width(520.dp),
        ) {
            Column(Modifier.padding(Dimens.SpaceMd)) {
                // Header
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
                ) {
                    Box(
                        modifier = Modifier
                            .size(36.dp)
                            .clip(RoundedCornerShape(Dimens.RadiusBase))
                            .background(QuireTheme.colors.accent.copy(alpha = 0.15f)),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            painter = painterResource(Res.drawable.ic_add),
                            contentDescription = null,
                            tint = QuireTheme.colors.accent,
                            modifier = Modifier.size(20.dp),
                        )
                    }
                    Column {
                        Text(
                            text = "Add Reference",
                            style = MaterialTheme.typography.titleLarge,
                            fontWeight = FontWeight.SemiBold,
                            color = QuireTheme.colors.fillPrimary,
                        )
                        Text(
                            text = "Import by DOI, enter details manually, or paste BibTeX",
                            style = MaterialTheme.typography.bodySmall,
                            color = QuireTheme.colors.fillSecondary,
                        )
                    }
                }

                Spacer(Modifier.height(Dimens.SpaceSm))

                PrimaryTabRow(
                    selectedTabIndex = selectedTab,
                    containerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                    modifier = Modifier.clip(RoundedCornerShape(Dimens.RadiusSmall)),
                ) {
                    Tab(
                        selected = selectedTab == 0,
                        onClick = { selectedTab = 0 },
                        text = { Text("Identifier") },
                    )
                    Tab(
                        selected = selectedTab == 1,
                        onClick = { selectedTab = 1 },
                        text = { Text("Manual") },
                    )
                    Tab(
                        selected = selectedTab == 2,
                        onClick = { selectedTab = 2 },
                        text = { Text("BibTeX") },
                    )
                }

                Spacer(Modifier.height(Dimens.SpaceMd))

                when (selectedTab) {
                    0 -> {
                        // Identifier Lookup
                        OutlinedTextField(
                            value = identifierInput,
                            onValueChange = { identifierInput = it },
                            label = { Text("DOI, arXiv ID, or PMID") },
                            placeholder = { Text("e.g. 10.1038/s41586-021-03819-2") },
                            singleLine = true,
                            modifier = Modifier.fillMaxWidth(),
                            colors = OutlinedTextFieldDefaults.colors(
                                focusedBorderColor = QuireTheme.colors.accent,
                            ),
                        )

                        Spacer(Modifier.height(Dimens.SpaceXs))

                        Button(
                            onClick = {
                                scope.launch {
                                    isLookingUp = true
                                    delay(400) // Simulated fast fetch
                                    isLookingUp = false
                                    lookupPreview = Reference(
                                        id = "custom-${System.currentTimeMillis()}",
                                        citeKey = "jumper2021alphafold",
                                        kind = "article-journal",
                                        title = "Highly accurate protein structure prediction with AlphaFold",
                                        creators = listOf(
                                            Creator("Jumper", "Jumper, John"),
                                            Creator("Hassabis", "Hassabis, Demis"),
                                        ),
                                        year = 2021,
                                        containerTitle = "Nature",
                                        volume = "596",
                                        issue = "7873",
                                        pages = "583–589",
                                        doi = identifierInput.trim(),
                                        abstract = "Neural networks predicting 3D structures of proteins with atomic accuracy.",
                                        tags = listOf("deep learning", "structural-biology", "proteins"),
                                        hasPdf = true,
                                        reading = ReadingStatus.Unread,
                                        dateAdded = "2026-10-20",
                                    )
                                }
                            },
                            enabled = identifierInput.isNotBlank() && !isLookingUp,
                            modifier = Modifier.align(Alignment.End),
                        ) {
                            if (isLookingUp) {
                                CircularProgressIndicator(
                                    modifier = Modifier.size(16.dp),
                                    strokeWidth = 2.dp,
                                    color = MaterialTheme.colorScheme.onPrimary,
                                )
                                Spacer(Modifier.width(Dimens.SpaceXs))
                            }
                            Icon(painterResource(Res.drawable.ic_auto_fix_high), contentDescription = null, modifier = Modifier.size(16.dp))
                            Spacer(Modifier.width(Dimens.SpaceXs))
                            Text("Fetch Metadata")
                        }

                        lookupPreview?.let { ref ->
                            Spacer(Modifier.height(Dimens.SpaceSm))
                            Card(
                                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHighest),
                                shape = RoundedCornerShape(Dimens.RadiusSmall),
                                modifier = Modifier.fillMaxWidth(),
                            ) {
                                Column(Modifier.padding(Dimens.SpaceSm)) {
                                    Text(ref.title, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
                                    Text(
                                        ref.creators.joinToString(", ") { it.displayName } + " (${ref.year})",
                                        style = MaterialTheme.typography.bodySmall,
                                        color = QuireTheme.colors.fillSecondary,
                                    )
                                    Text(
                                        ref.containerTitle.orEmpty() + " • " + ref.doi.orEmpty(),
                                        style = MaterialTheme.typography.bodySmall,
                                        color = QuireTheme.colors.accent,
                                    )
                                }
                            }
                        }
                    }
                    1 -> {
                        // Manual Entry Form
                        Column(verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                            OutlinedTextField(
                                value = title,
                                onValueChange = { title = it },
                                label = { Text("Title *") },
                                singleLine = true,
                                modifier = Modifier.fillMaxWidth(),
                            )
                            Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                                OutlinedTextField(
                                    value = authors,
                                    onValueChange = { authors = it },
                                    label = { Text("Authors (e.g. Smith, John; Doe, Jane)") },
                                    singleLine = true,
                                    modifier = Modifier.weight(2f),
                                )
                                OutlinedTextField(
                                    value = year,
                                    onValueChange = { year = it },
                                    label = { Text("Year") },
                                    singleLine = true,
                                    modifier = Modifier.weight(1f),
                                )
                            }
                            Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                                OutlinedTextField(
                                    value = containerTitle,
                                    onValueChange = { containerTitle = it },
                                    label = { Text("Journal / Publication") },
                                    singleLine = true,
                                    modifier = Modifier.weight(1f),
                                )
                                OutlinedTextField(
                                    value = doi,
                                    onValueChange = { doi = it },
                                    label = { Text("DOI") },
                                    singleLine = true,
                                    modifier = Modifier.weight(1f),
                                )
                            }
                            OutlinedTextField(
                                value = tagsInput,
                                onValueChange = { tagsInput = it },
                                label = { Text("Tags (comma separated)") },
                                placeholder = { Text("neuroscience, review, methods") },
                                singleLine = true,
                                modifier = Modifier.fillMaxWidth(),
                            )
                        }
                    }
                    2 -> {
                        // BibTeX Paste
                        OutlinedTextField(
                            value = bibtexInput,
                            onValueChange = { bibtexInput = it },
                            label = { Text("Paste BibTeX Entry") },
                            placeholder = { Text("@article{key,\n  title={...},\n  author={...}\n}") },
                            modifier = Modifier.fillMaxWidth().height(160.dp),
                            textStyle = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
                        )
                    }
                }

                Spacer(Modifier.height(Dimens.SpaceMd))

                // Bottom actions
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    TextButton(onClick = onDismiss) {
                        Text("Cancel")
                    }
                    Spacer(Modifier.width(Dimens.SpaceXs))
                    Button(
                        onClick = {
                            val refToAdd = when (selectedTab) {
                                0 -> lookupPreview
                                1 -> {
                                    val parsedAuthors = authors.split(';').map { it.trim() }.filter { it.isNotBlank() }
                                        .map { Creator(sortName = it.substringBefore(','), displayName = it) }
                                    val cite = (parsedAuthors.firstOrNull()?.sortName?.lowercase() ?: "ref") + year
                                    Reference(
                                        id = "custom-${System.currentTimeMillis()}",
                                        citeKey = cite,
                                        kind = "article-journal",
                                        title = title.ifBlank { "Untitled Reference" },
                                        creators = parsedAuthors,
                                        year = year.toIntOrNull(),
                                        containerTitle = containerTitle,
                                        doi = doi,
                                        tags = tagsInput.split(',').map { it.trim() }.filter { it.isNotBlank() },
                                        hasPdf = true,
                                        reading = ReadingStatus.Unread,
                                        dateAdded = "2026-10-20",
                                    )
                                }
                                2 -> {
                                    Reference(
                                        id = "bibtex-${System.currentTimeMillis()}",
                                        citeKey = "importedBibtex",
                                        kind = "article-journal",
                                        title = "Imported BibTeX Document",
                                        creators = listOf(Creator("Author", "Author, Example")),
                                        year = 2026,
                                        tags = listOf("bibtex"),
                                        hasPdf = true,
                                        reading = ReadingStatus.Unread,
                                        dateAdded = "2026-10-20",
                                    )
                                }
                                else -> null
                            }
                            if (refToAdd != null) {
                                onAdd(refToAdd)
                                onDismiss()
                            }
                        },
                        colors = ButtonDefaults.buttonColors(
                            containerColor = QuireTheme.colors.accent,
                        ),
                        enabled = when (selectedTab) {
                            0 -> lookupPreview != null
                            1 -> title.isNotBlank()
                            2 -> bibtexInput.isNotBlank()
                            else -> false
                        },
                    ) {
                        Text("Add to Library")
                    }
                }
            }
        }
    }
}
