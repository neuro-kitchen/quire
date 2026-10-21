package dev.neurokitchen.quire.di

import dev.neurokitchen.quire.presentation.screens.tools.ToolsViewModel
import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.data.repository.SamplePdfRepository
import dev.neurokitchen.quire.data.repository.SamplePreferencesRepository
import dev.neurokitchen.quire.data.repository.SampleReaderRepository
import dev.neurokitchen.quire.data.repository.SampleSettingsRepository
import dev.neurokitchen.quire.domain.repository.AttachmentRepository
import dev.neurokitchen.quire.domain.repository.CurationRepository
import dev.neurokitchen.quire.domain.repository.ExchangeRepository
import dev.neurokitchen.quire.domain.repository.LibrariesRepository
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.PdfRepository
import dev.neurokitchen.quire.domain.repository.PreferencesRepository
import dev.neurokitchen.quire.domain.repository.ReaderRepository
import dev.neurokitchen.quire.domain.repository.SearchRepository
import dev.neurokitchen.quire.domain.repository.SettingsRepository
import dev.neurokitchen.quire.domain.repository.SyncRepository
import dev.neurokitchen.quire.presentation.desktop.DesktopViewModel
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.reader.ReaderViewModel
import dev.neurokitchen.quire.presentation.screens.settings.SettingsViewModel
import org.koin.core.module.Module
import org.koin.core.module.dsl.viewModel
import org.koin.core.module.dsl.viewModelOf
import dev.neurokitchen.quire.presentation.components.export_dialog.ExportDialogViewModel
import dev.neurokitchen.quire.presentation.components.export_dialog.ExportTarget
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
    single<PreferencesRepository> { SamplePreferencesRepository() }
    // No engine on these platforms yet: every call answers "needs the engine".
    single<AttachmentRepository> { object : AttachmentRepository {} }
    single<SearchRepository> { object : SearchRepository {} }
    single<CurationRepository> { object : CurationRepository {} }
    single<ExchangeRepository> { object : ExchangeRepository {} }
    single<LibrariesRepository> { object : LibrariesRepository {} }
    single<SyncRepository> { object : SyncRepository {} }
}

val viewModelModule = module {
    viewModelOf(::LibraryViewModel)
    viewModelOf(::SettingsViewModel)
    viewModelOf(::DesktopViewModel)
    viewModelOf(::ToolsViewModel)
    // One per open paper: its reference ID and title come as parameters.
    viewModel { (id: String, title: String) -> ReaderViewModel(id, title, get(), get(), preferences = getOrNull()) }
    // One per export dialog: what it exports comes as a parameter.
    viewModel { (target: ExportTarget) -> ExportDialogViewModel(target, get()) }
}

/** Everything the app needs, with `data` supplying the repositories. */
fun appModules(data: Module): List<Module> = listOf(data, viewModelModule)
