package dev.neurokitchen.quire

import android.app.Application
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

class QuireApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        // The real library through the engine (the sample data is for tests and previews).
        initKoin(data = androidEngineModule(this)) { androidContext(this@QuireApplication) }
        seedSamplePapersIfEmpty()
    }

    private fun seedSamplePapersIfEmpty() {
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

