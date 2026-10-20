package dev.neurokitchen.quire

import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.adaptive.navigationsuite.NavigationSuiteScaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.navigation.Destination
import dev.neurokitchen.quire.presentation.screens.library.LibraryRoute
import dev.neurokitchen.quire.presentation.screens.settings.SettingsRoute
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/** The phone and tablet app; Android and iOS show it after `initKoin` (the desktop uses `DesktopApp`). */
@Composable
fun App() {
    QuireTheme {
        var current by rememberSaveable { mutableStateOf(Destination.Library) }
        val labels = Destination.entries.associateWith { stringResource(it.label) }
        NavigationSuiteScaffold(
            navigationSuiteItems = {
                Destination.entries.forEach { destination ->
                    val label = labels.getValue(destination)
                    item(
                        selected = destination == current,
                        onClick = { current = destination },
                        icon = { Icon(painterResource(destination.icon), contentDescription = label) },
                        label = { Text(label) },
                    )
                }
            },
        ) {
            when (current) {
                Destination.Library -> LibraryRoute()
                Destination.Settings -> SettingsRoute()
            }
        }
    }
}
