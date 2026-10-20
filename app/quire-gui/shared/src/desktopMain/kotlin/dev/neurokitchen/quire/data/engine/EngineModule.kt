package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.PdfRepository
import dev.neurokitchen.quire.domain.repository.ReaderRepository
import dev.neurokitchen.quire.domain.repository.SettingsRepository
import dev.neurokitchen.quire.ffi.openLibrary
import org.koin.dsl.module

/** The desktop's data: the default library through the engine, with the usual config file. */
val engineDataModule = module {
    single { EngineSession(open = { openLibrary(null, null) }) }
    single<LibraryRepository> { EngineLibraryRepository(get()) }
    single<ReaderRepository> { EngineReaderRepository(get()) }
    single<PdfRepository> { EnginePdfRepository(EnginePdfRepository::bundledDir) }
    single<SettingsRepository> { EngineSettingsRepository() }
}
