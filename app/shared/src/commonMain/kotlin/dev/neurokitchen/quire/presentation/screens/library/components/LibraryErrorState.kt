package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.library_error
import org.jetbrains.compose.resources.stringResource

/** Shown instead of the list when the engine could not answer (the message is the engine's). */
@Composable
fun LibraryErrorState(message: String, modifier: Modifier = Modifier) {
    Column(
        modifier.fillMaxSize().padding(Dimens.SpaceLg),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(stringResource(Res.string.library_error), style = MaterialTheme.typography.titleMedium, color = QuireTheme.colors.accentRed)
        Text(
            message,
            style = MaterialTheme.typography.bodyMedium,
            color = QuireTheme.colors.fillSecondary,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(top = Dimens.SpaceXs),
        )
    }
}
