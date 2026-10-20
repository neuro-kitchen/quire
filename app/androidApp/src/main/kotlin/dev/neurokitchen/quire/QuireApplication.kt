package dev.neurokitchen.quire

import android.app.Application
import dev.neurokitchen.quire.di.initKoin
import org.koin.android.ext.koin.androidContext

class QuireApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        initKoin { androidContext(this@QuireApplication) }
    }
}
