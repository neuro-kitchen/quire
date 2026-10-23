package dev.neurokitchen.quire.data.engine

import android.content.Context
import android.system.Os
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
import dev.neurokitchen.quire.domain.repository.NotificationsRepository
import dev.neurokitchen.quire.domain.repository.SyncRepository
import dev.neurokitchen.quire.ffi.openLibraryAt
import org.koin.core.module.Module
import org.koin.dsl.module
import java.io.File

/**
 * Android's data: the engine (quire-ffi, built per ABI with cargo-ndk) on a library in the app's
 * private storage, with its config and the GUI's preferences beside it. Secrets go to a file there
 * too (Android has no Secret Service); the app sandbox keeps it private.
 */
fun androidEngineModule(context: Context): Module {
    val base = context.filesDir
    val config = File(base, "config.toml").path
    // Read by the engine: no keyring (a file instead), and a home directory (Android apps have
    // none; the engine's defaults, e.g. the downloads folder, start from it).
    Os.setenv("QUIRE_NO_KEYRING", "1", true)
    Os.setenv("HOME", base.path, true)
    AndroidTls.initialize(context)
    return module {
        single { EngineSession(open = { openLibraryAt(File(base, "library").path, config) }) }
        single<PreferencesRepository> { FilePreferencesRepository(File(base, "gui-preferences.properties")) }
        single<LibraryRepository> { EngineLibraryRepository(get()) }
        single<ReaderRepository> { EngineReaderRepository(get()) }
        single<AttachmentRepository> { EngineAttachmentRepository(get()) }
        single<SearchRepository> { EngineSearchRepository(get()) }
        single<CurationRepository> { EngineCurationRepository(get()) }
        single<ExchangeRepository> { EngineExchangeRepository(get()) }
        single<LibrariesRepository> { EngineLibrariesRepository(get(), config) }
        single<SyncRepository> { EngineSyncRepository(get()) }
        single<NotificationsRepository> { EngineNotificationsRepository(get()) }
        single<PdfRepository> { EnginePdfRepository() }
        single<SettingsRepository> { EngineSettingsRepository(config) }
    }
}
