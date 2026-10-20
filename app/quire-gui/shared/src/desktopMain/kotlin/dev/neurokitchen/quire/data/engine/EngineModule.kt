package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.repository.AttachmentRepository
import dev.neurokitchen.quire.domain.repository.CurationRepository
import dev.neurokitchen.quire.domain.repository.ExchangeRepository
import dev.neurokitchen.quire.domain.repository.LibrariesRepository
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.PdfRepository
import dev.neurokitchen.quire.domain.repository.PreferencesRepository
import dev.neurokitchen.quire.domain.repository.ReaderRepository
import dev.neurokitchen.quire.domain.repository.SearchRepository
import dev.neurokitchen.quire.domain.repository.SyncRepository
import dev.neurokitchen.quire.domain.repository.SettingsRepository
import dev.neurokitchen.quire.ffi.openLibrary
import org.koin.dsl.module

/** The desktop's data: the default library through the engine, with the usual config file. */
val engineDataModule = module {
    single { EngineSession(open = { openLibrary(null, null) }) }
    single<PreferencesRepository> { FilePreferencesRepository() }
    single<LibraryRepository> { EngineLibraryRepository(get()) }
    single<ReaderRepository> { EngineReaderRepository(get()) }
    single<AttachmentRepository> { EngineAttachmentRepository(get()) }
    single<SearchRepository> { EngineSearchRepository(get()) }
    single<CurationRepository> { EngineCurationRepository(get()) }
    single<ExchangeRepository> { EngineExchangeRepository(get()) }
    single<LibrariesRepository> { EngineLibrariesRepository(get()) }
    single<SyncRepository> { EngineSyncRepository(get()) }
    single<PdfRepository> { EnginePdfRepository() }
    single<SettingsRepository> { EngineSettingsRepository() }
}
