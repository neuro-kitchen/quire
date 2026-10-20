package dev.neurokitchen.quire

import dev.neurokitchen.quire.di.appModules
import dev.neurokitchen.quire.di.sampleDataModule
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.settings.SettingsViewModel
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin
import kotlin.test.AfterTest
import kotlin.test.Test

class KoinGraphTest {
    @AfterTest
    fun tearDown() = stopKoin()

    @Test
    fun everyViewModelResolves() {
        val koin = startKoin { modules(appModules(sampleDataModule)) }.koin
        koin.get<LibraryViewModel>()
        koin.get<SettingsViewModel>()
    }
}
