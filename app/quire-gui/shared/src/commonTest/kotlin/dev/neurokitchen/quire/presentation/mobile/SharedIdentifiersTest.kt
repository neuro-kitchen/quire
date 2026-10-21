package dev.neurokitchen.quire.presentation.mobile

import dev.neurokitchen.quire.presentation.mobile.share.sharedIdentifiers
import kotlin.test.Test
import kotlin.test.assertEquals

class SharedIdentifiersTest {
    /** A browser shares "title + link"; mail and chat share whole sentences: keep what identifies. */
    @Test
    fun identifiersArePickedOutOfSharedText() {
        assertEquals(listOf("https://www.nature.com/articles/nature03721"), sharedIdentifiers("Microstructure of a spatial map in the entorhinal cortex\nhttps://www.nature.com/articles/nature03721"))
        assertEquals(listOf("10.1016/j.smim.2007.11.004"), sharedIdentifiers("Have a look: doi 10.1016/j.smim.2007.11.004."))
        assertEquals(listOf("arXiv:1706.03762", "https://doi.org/10.1038/nature14539"), sharedIdentifiers("arXiv:1706.03762 and https://doi.org/10.1038/nature14539, both"))
        assertEquals(listOf("PMID: 18160298"), sharedIdentifiers("PMID: 18160298"))
    }

    @Test
    fun textWithoutAnIdentifierIsKeptAsItIs() {
        assertEquals(listOf("Attention is all you need"), sharedIdentifiers("  Attention is all you need  "))
        assertEquals(emptyList(), sharedIdentifiers("   "))
    }
}
