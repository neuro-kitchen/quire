package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.ffi.ChangeListener
import dev.neurokitchen.quire.ffi.QuireLibrary
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.conflate
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onStart

/**
 * The open library every engine repository shares. It opens on first use and again on [reopen]
 * (after the settings changed) or [switchTo] (another library); [reloading] flows query again after each change the engine
 * reports (edits, sync, background jobs) and switch to the new library after a reopen.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class EngineSession(
    private var open: suspend () -> QuireLibrary,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) {
    private val current = MutableStateFlow(opening())

    private fun opening(): Deferred<QuireLibrary> = scope.async(start = CoroutineStart.LAZY) { open() }

    suspend fun library(): QuireLibrary = current.value.await()

    suspend fun reopen() {
        val old = current.value
        current.value = opening()
        if (old.isCompleted && !old.isCancelled) runCatching { old.getCompleted().close() }
    }

    /** Opens another library through [open] from now on; open flows switch to it. */
    suspend fun switchTo(open: suspend () -> QuireLibrary) {
        this.open = open
        reopen()
    }

    /** `load` now and after every change, the latest result winning. */
    fun <T> reloading(load: suspend (QuireLibrary) -> T): Flow<T> =
        current.flatMapLatest { opening ->
            changes(opening).onStart { emit(Unit) }.conflate().mapLatest { load(opening.await()) }
        }

    private fun changes(opening: Deferred<QuireLibrary>): Flow<Unit> = callbackFlow {
        val subscription = opening.await().subscribe(object : ChangeListener {
            override fun onChange() {
                trySend(Unit)
            }
        })
        awaitClose {
            subscription.cancel()
            subscription.close()
        }
    }
}
