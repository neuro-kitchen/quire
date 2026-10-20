package dev.neurokitchen.quire.di

import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.data.repository.SamplePdfRepository
import dev.neurokitchen.quire.data.repository.SampleReaderRepository
import dev.neurokitchen.quire.data.repository.SampleSettingsRepository
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.PdfRepository
import dev.neurokitchen.quire.domain.repository.ReaderRepository
import dev.neurokitchen.quire.domain.repository.SettingsRepository
import dev.neurokitchen.quire.presentation.desktop.DesktopViewModel
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.reader.ReaderViewModel
import dev.neurokitchen.quire.presentation.screens.settings.SettingsViewModel
import org.koin.core.module.Module
import org.koin.core.module.dsl.viewModel
import org.koin.core.module.dsl.viewModelOf
import org.koin.dsl.module

/** Sample data: tests, previews, and platforms without the engine bridge yet (Android, iOS). */
val sampleDataModule = module {
    single<LibraryRepository> {
        SampleLibraryRepository(
            references = SampleLibraryRepository.REFERENCES + SampleLibraryRepository.EXTENDED_REFERENCES,
        )
    }
    single<SettingsRepository> { SampleSettingsRepository() }
    single<ReaderRepository> {
        SampleReaderRepository(
            initialHighlights = SampleReaderRepository.SAMPLE_HIGHLIGHTS,
            initialNotes = SampleReaderRepository.SAMPLE_NOTES,
            fakePdfs = true,
        )
    }
    single<PdfRepository> { SamplePdfRepository() }
}

val viewModelModule = module {
    viewModelOf(::LibraryViewModel)
    viewModelOf(::SettingsViewModel)
    viewModelOf(::DesktopViewModel)
    // One per open paper: its reference ID and title come as parameters.
    viewModel { (id: String, title: String) -> ReaderViewModel(id, title, get(), get()) }
}

/** Everything the app needs, with `data` supplying the repositories. */
fun appModules(data: Module): List<Module> = listOf(data, viewModelModule)
