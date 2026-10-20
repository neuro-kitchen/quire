package dev.neurokitchen.quire.presentation.screens.library

import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview
import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme

private val cards = SampleLibraryRepository.REFERENCES.map { ReferenceCardState.from(it) }

@Preview(widthDp = 1280, heightDp = 800)
@Composable
private fun LibraryDesktopPreview() {
    QuireTheme(desktop = true) {
        Surface { LibraryScreen(LibraryState(isLoading = false, references = cards), onEvent = {}) }
    }
}

@Preview(widthDp = 400, heightDp = 800)
@Composable
private fun LibraryPhonePreview() {
    QuireTheme { Surface { LibraryScreen(LibraryState(isLoading = false, references = cards), onEvent = {}) } }
}

@Preview(widthDp = 400, heightDp = 800)
@Composable
private fun LibraryNoMatchPreview() {
    QuireTheme { Surface { LibraryScreen(LibraryState(isLoading = false, query = "zebrafish"), onEvent = {}) } }
}
