package dev.neurokitchen.quire.presentation.screens.reader.components

import dev.neurokitchen.quire.domain.model.BlockKind
import dev.neurokitchen.quire.domain.model.TextBlock
import dev.neurokitchen.quire.domain.model.Word
import kotlin.test.Test
import kotlin.test.assertEquals

class ReaderItemsTest {
    private fun block(kind: BlockKind, vararg words: String) = TextBlock(kind, words.map { Word(it) })

    /**
     * Table rows were each drawn as one line, their cells joined by spaces. A run of rows is now one
     * table item; find and highlight jumps go through the block → item map.
     */
    @Test
    fun aRunOfTableRowsIsOneItem() {
        val blocks = listOf(
            block(BlockKind.Heading(2), "Results"),
            block(BlockKind.TableRow, "Probe", "ΔT (K)"),
            block(BlockKind.TableRow, "NV centre", "0.044"),
            block(BlockKind.TableRow, "Gold nanoparticle", "0.1"),
            block(BlockKind.Paragraph, "As the table shows"),
            block(BlockKind.TableRow, "lone", "row"),
        )
        val items = readerItems(blocks)
        assertEquals(listOf(ReaderItem(0), ReaderItem(1, 3), ReaderItem(4), ReaderItem(5)), items)
        assertEquals(listOf(0, 1, 1, 1, 2, 3), itemOfBlock(items, blocks.size).toList())
    }
}
