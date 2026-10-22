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
import dev.neurokitchen.quire.domain.repository.NotificationsRepository
import dev.neurokitchen.quire.domain.repository.SyncRepository
import dev.neurokitchen.quire.domain.repository.SettingsRepository
import dev.neurokitchen.quire.ffi.QuireLibrary
import dev.neurokitchen.quire.ffi.openLibrary
import dev.neurokitchen.quire.ffi.openLibraryAt
import org.koin.dsl.module

/** The desktop's data: the default library through the engine, with the usual config file. */
val engineDataModule = module {
    single { EngineSession(open = ::openChosenLibrary) }
    single<PreferencesRepository> { FilePreferencesRepository() }
    single<LibraryRepository> { EngineLibraryRepository(get()) }
    single<ReaderRepository> { EngineReaderRepository(get()) }
    single<AttachmentRepository> { EngineAttachmentRepository(get()) }
    single<SearchRepository> { EngineSearchRepository(get()) }
    single<CurationRepository> { EngineCurationRepository(get()) }
    single<ExchangeRepository> { EngineExchangeRepository(get()) }
    single<LibrariesRepository> { EngineLibrariesRepository(get()) }
    single<SyncRepository> { EngineSyncRepository(get()) }
    single<NotificationsRepository> { EngineNotificationsRepository(get()) }
    single<PdfRepository> { EnginePdfRepository() }
    single<SettingsRepository> { EngineSettingsRepository() }
}

/**
 * The library to open, as the command line chooses it: the folder in `QUIRE_LIBRARY`, else the
 * registered library named in `QUIRE_LIB`, else the default one. (Tests and side-by-side copies
 * rely on it: without it the app would open the person's own library.)
 */
private suspend fun openChosenLibrary(): QuireLibrary {
    System.getenv("QUIRE_LIBRARY")?.takeIf { it.isNotBlank() }?.let { return openLibraryAt(it, null) }
    return openLibrary(System.getenv("QUIRE_LIB")?.takeIf { it.isNotBlank() }, null)
}
