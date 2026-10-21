package dev.neurokitchen.quire

import android.app.Application
import dev.neurokitchen.quire.data.engine.androidEngineModule
import dev.neurokitchen.quire.di.initKoin
import org.koin.android.ext.koin.androidContext

class QuireApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        // The real library through the engine (the sample data is for tests and previews).
        initKoin(data = androidEngineModule(this)) { androidContext(this@QuireApplication) }
    }
}
