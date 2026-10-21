package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.repository.ReaderLayout
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import kotlinx.coroutines.runBlocking
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals

class FilePreferencesRepositoryTest {
    @Test
    fun theReaderLayoutIsKeptBetweenRuns() = runBlocking {
        val file = Files.createTempDirectory("quire-prefs").resolve("gui.properties").toFile()
        val layout = ReaderLayout(annotationsWidth = 300, notesWidth = 220, drawerWidth = 400, drawerOnLeft = true, two = listOf(0.35f, 0.65f), three = listOf(0.5f, 0.3f, 0.2f))
        FilePreferencesRepository(file).apply {
            setReaderTheme(ReaderTheme.Sepia)
            setReaderLayout(layout)
        }
        val again = FilePreferencesRepository(file).preferences.value
        assertEquals(layout, again.readerLayout)
        assertEquals(ReaderTheme.Sepia, again.readerTheme, "the other settings stay")
    }

    @Test
    fun aDamagedLayoutFallsBackToTheDefaults() = runBlocking {
        val file = Files.createTempDirectory("quire-prefs").resolve("gui.properties").toFile()
        file.writeText("reader.layout.annotations=wide\nreader.layout.two=0.5;x\n")
        assertEquals(ReaderLayout(), FilePreferencesRepository(file).preferences.value.readerLayout)
    }
}
