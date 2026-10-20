package dev.neurokitchen.quire.presentation.components.reference_card

import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview
import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme

@Preview
@Composable
private fun ReferenceCardPreview() {
    QuireTheme {
        ReferenceCard(ReferenceCardState.from(SampleLibraryRepository.REFERENCES[0]), onEvent = {})
    }
}

@Preview
@Composable
private fun ReferenceCardSelectedPreview() {
    QuireTheme {
        ReferenceCard(ReferenceCardState.from(SampleLibraryRepository.REFERENCES[6], isSelected = true), onEvent = {})
    }
}
