package dev.neurokitchen.quire

import android.app.Application
import android.content.pm.ApplicationInfo
import dev.neurokitchen.quire.data.engine.androidEngineModule
import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.di.initKoin
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import org.koin.android.ext.android.get
import org.koin.android.ext.koin.androidContext
import java.io.File

class QuireApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        // The real library through the engine (the sample data is for tests and previews).
        initKoin(data = androidEngineModule(this)) { androidContext(this@QuireApplication) }
        seedSamplePapersIfEmpty()
    }

    /**
     * Sample papers for UI work, only when asked: a debuggable build with the marker file
     * (`adb shell run-as dev.neurokitchen.quire touch files/seed-samples`). Never by default: they
     * would land in the real library and sync to the person's other devices, and a new device must
     * start empty to join a library through the hub.
     */
    private fun seedSamplePapersIfEmpty() {
        val debuggable = applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE != 0
        if (!debuggable || !File(filesDir, "seed-samples").exists()) return
        val library = get<LibraryRepository>()
        CoroutineScope(Dispatchers.IO).launch {
            val existing = runCatching { library.references(LibraryQuery()).first() }.getOrDefault(emptyList())
            if (existing.size <= 2) {
                runCatching { library.createProject("Thesis", null) }
                runCatching { library.createProject("Chapter 2", "thesis") }
                runCatching { library.createProject("Deep Learning", null) }
                runCatching { library.createProject("Reading Group", null) }

                val allSamples = SampleLibraryRepository.REFERENCES + SampleLibraryRepository.EXTENDED_REFERENCES
                val existingKeys = existing.map { it.citeKey }.toSet()
                for (ref in allSamples) {
                    if (ref.citeKey !in existingKeys) {
                        runCatching { library.addReference(ref) }
                    }
                }
            }
        }
    }
}

