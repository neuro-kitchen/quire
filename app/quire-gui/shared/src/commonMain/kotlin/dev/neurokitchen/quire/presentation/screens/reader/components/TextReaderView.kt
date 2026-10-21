package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.foundation.layout.calculateEndPadding
import androidx.compose.foundation.layout.calculateStartPadding
import androidx.compose.ui.platform.LocalLayoutDirection
import dev.neurokitchen.quire.presentation.screens.reader.TextMatch
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.domain.model.BlockKind
import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.domain.model.PaperTab
import dev.neurokitchen.quire.domain.model.PaperTabKind
import dev.neurokitchen.quire.domain.model.TextBlock
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import dev.neurokitchen.quire.presentation.designsystem.readerColors
import dev.neurokitchen.quire.presentation.mapping.loadImageFile
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.tab_abstract
import dev.neurokitchen.quire.resources.tab_equations
import dev.neurokitchen.quire.resources.tab_figures
import dev.neurokitchen.quire.resources.tab_front
import dev.neurokitchen.quire.resources.tab_paper
import dev.neurokitchen.quire.resources.tab_references
import dev.neurokitchen.quire.resources.tab_tables
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

/**
 * The paper's text, one tab at a time (abstract, paper, figures, …), in a readable column.
 * Supports font selection (Serif, Sans, Mono), font sizing, 6 reader themes,
 * and bottom floating highlight palette.
 */
@Composable
fun TextReaderView(
    tabs: List<PaperTab>,
    selectedTab: Int,
    highlights: List<Highlight>,
    focused: String?,
    onTabClick: (Int) -> Unit,
    onSelection: (Pair<String, String>?) -> Unit,
    loadFigure: suspend (String) -> String?,
    modifier: Modifier = Modifier,
    readerTheme: ReaderTheme = ReaderTheme.Dark,
    readerFont: ReaderFont = ReaderFont.Serif,
    fontSizeSp: Int = 16,
    onHighlight: (quote: String, prefix: String, color: String) -> Unit = { _, _, _ -> },
    onAddNote: (body: String) -> Unit = {},
    /** Find matches in this tab, and the one to bring into view. */
    findMatches: List<TextMatch> = emptyList(),
    currentFind: Int = 0,
    onProgressChange: (currentBlock: Int, totalBlocks: Int) -> Unit = { _, _ -> },
    onTapPage: () -> Unit = {},
    /** Room for bars drawn over the text (the phone's frosted bars): the text scrolls under them. */
    contentPadding: PaddingValues = PaddingValues(),
    /** A block to bring to the top (the phone's progress slider); `null`: none. */
    jumpToBlock: Int? = null,
) {
    val tab = tabs.getOrNull(selectedTab) ?: return
    // Item 0 of the list is the tabs row (empty when the tabs sit above it): block items start at 1.
    val list = rememberLazyListState()
    val focusedQuote = highlights.firstOrNull { it.id == focused }?.quote
    val themeColors = readerColors(readerTheme)
    var localSelection by remember(selectedTab) { mutableStateOf<Pair<String, String>?>(null) }

    // Consecutive table rows show as one table: list items are not blocks one to one.
    val items = remember(tab) { readerItems(tab.blocks) }
    val itemOfBlock = remember(items, tab) { itemOfBlock(items, tab.blocks.size) }
    val firstVisibleBlock by remember(items, list) {
        derivedStateOf { (items.getOrNull((list.firstVisibleItemIndex - 1).coerceAtLeast(0))?.first ?: 0) + 1 }
    }
    LaunchedEffect(firstVisibleBlock, tab.blocks.size) {
        onProgressChange(firstVisibleBlock, tab.blocks.size)
    }

    val current = findMatches.getOrNull(currentFind)
    LaunchedEffect(current) { if (current != null) list.animateScrollToItem(itemOfBlock[current.block] + 1) }

    LaunchedEffect(focusedQuote, selectedTab) {
        val index = focusedQuote?.let { q ->
            val clean = q.replace(Regex("\\s+"), " ").trim().lowercase()
            tab.blocks.indexOfFirst { block ->
                val blockText = block.text.replace(Regex("\\s+"), " ").lowercase()
                clean in blockText || blockText in clean
            }
        } ?: -1
        if (index >= 0) list.animateScrollToItem(itemOfBlock[index] + 1)
    }

    LaunchedEffect(jumpToBlock) { if (jumpToBlock != null) list.scrollToItem(itemOfBlock.getOrElse(jumpToBlock) { 0 } + 1) }
    val layoutDirection = LocalLayoutDirection.current
    val focusManager = LocalFocusManager.current

    Column(modifier.fillMaxSize().background(themeColors.background)) {
        val tabRow: @Composable () -> Unit = {
            Row(
                Modifier
                    .fillMaxWidth()
                    .horizontalScroll(rememberScrollState())
                    .background(themeColors.surface)
                    .padding(Dimens.SpaceXs),
                horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceMin),
            ) {
                tabs.forEachIndexed { index, t ->
                    val label = stringResource(t.kind.label) + (t.count?.let { " ($it)" } ?: "")
                    TabChip(label, selected = index == selectedTab) { onTabClick(index) }
                }
            }
        }
        // Under bars drawn over the text, the tabs scroll with it; otherwise they stay at the top.
        val tabsInList = contentPadding.calculateTopPadding() > 0.dp
        if (!tabsInList) tabRow()

        Box(
            Modifier
                .fillMaxSize()
                .pointerInput(Unit) {
                    detectTapGestures { onTapPage() }
                },
            contentAlignment = Alignment.TopCenter,
        ) {
            LazyColumn(
                state = list,
                verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
                contentPadding = PaddingValues(
                    start = Dimens.SpaceLg + contentPadding.calculateStartPadding(layoutDirection),
                    end = Dimens.SpaceLg + contentPadding.calculateEndPadding(layoutDirection),
                    top = Dimens.SpaceMd + contentPadding.calculateTopPadding(),
                    bottom = Dimens.SpaceMd + contentPadding.calculateBottomPadding(),
                ),
                modifier = Modifier.widthIn(max = ReadingWidth).fillMaxSize(),
            ) {
                // Item 0: the tabs when they scroll with the text (jumps count it).
                item { if (tabsInList) tabRow() }
                items(items) { item ->
                    val blockIndex = item.first
                    val block = tab.blocks[blockIndex]
                    when (val kind = block.kind) {
                        is BlockKind.Image -> Figure(kind.url, loadFigure)
                        BlockKind.TableRow -> PaperTable(
                            rows = tab.blocks.subList(item.first, item.last + 1).map { row -> row.words.map { it.text } },
                            textStyle = style(BlockKind.Paragraph, readerFont, (fontSizeSp * 0.9f).toInt()),
                            textColor = themeColors.text,
                            lineColor = themeColors.text.copy(alpha = 0.18f),
                            stripeColor = themeColors.text.copy(alpha = 0.04f),
                        )
                        else -> SelectableBlock(
                            block = block,
                            highlights = highlights,
                            readerFont = readerFont,
                            fontSizeSp = fontSizeSp,
                            textColor = themeColors.text,
                            finds = findMatches.filter { it.block == blockIndex },
                            currentFind = current?.takeIf { it.block == blockIndex },
                            onSelection = {
                                localSelection = it
                                onSelection(it)
                            },
                        )
                    }
                }
            }

            // Bottom-anchored Floating Highlight Action Palette
            localSelection?.let { sel ->
                SelectionActionPalette(
                    visible = true,
                    onHighlight = { colorName ->
                        onHighlight(sel.first, sel.second, colorName)
                        localSelection = null
                        // Ends the selection (and the system's text toolbar with it).
                        focusManager.clearFocus()
                        onSelection(null)
                    },
                    onAddNote = {
                        onAddNote(sel.first)
                        localSelection = null
                        // Ends the selection (and the system's text toolbar with it).
                        focusManager.clearFocus()
                        onSelection(null)
                    },
                    onCopy = {
                        localSelection = null
                        // Ends the selection (and the system's text toolbar with it).
                        focusManager.clearFocus()
                        onSelection(null)
                    },
                    onCopyCitation = {
                        localSelection = null
                        // Ends the selection (and the system's text toolbar with it).
                        focusManager.clearFocus()
                        onSelection(null)
                    },
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .padding(bottom = Dimens.SpaceLg + contentPadding.calculateBottomPadding()),
                )
            }
        }
    }
}

/** A comfortable line length for reading (about 75 characters at 16 px). */
private val ReadingWidth = 760.dp

@Composable
private fun SelectableBlock(
    block: TextBlock,
    highlights: List<Highlight>,
    readerFont: ReaderFont,
    fontSizeSp: Int,
    textColor: Color,
    onSelection: (Pair<String, String>?) -> Unit,
    finds: List<TextMatch> = emptyList(),
    currentFind: TextMatch? = null,
) {
    val colors = QuireTheme.colors
    val text = remember(block, highlights, finds, currentFind, colors.accent) { annotate(block, highlights, colors, finds, currentFind) }
    var value by remember(text) { mutableStateOf(TextFieldValue(text.text)) }
    BasicTextField(
        value = value,
        visualTransformation = { TransformedText(text, OffsetMapping.Identity) },
        onValueChange = { next ->
            value = value.copy(selection = next.selection)
            val range = next.selection
            onSelection(
                if (range.collapsed) null
                else text.text.substring(range.min, range.max) to text.text.substring(0, range.min),
            )
        },
        readOnly = true,
        textStyle = style(block.kind, readerFont, fontSizeSp).copy(color = textColor),
        cursorBrush = SolidColor(colors.accent),
        modifier = Modifier.fillMaxWidth().padding(start = if (block.kind == BlockKind.ListItem) Dimens.SpaceMd else 0.dp),
    )
}

@Composable
private fun style(kind: BlockKind, font: ReaderFont, fontSizeSp: Int): TextStyle {
    val fontFamily = when (font) {
        ReaderFont.Serif -> FontFamily.Serif
        ReaderFont.Sans -> FontFamily.Default
        ReaderFont.Mono -> FontFamily.Monospace
    }
    val base = TextStyle(
        fontFamily = fontFamily,
        fontSize = fontSizeSp.sp,
        lineHeight = (fontSizeSp * 1.6f).sp,
    )
    return when (kind) {
        is BlockKind.Heading -> when (kind.level) {
            1 -> base.copy(fontSize = (fontSizeSp * 1.5f).sp, lineHeight = (fontSizeSp * 2.0f).sp, fontWeight = FontWeight.SemiBold)
            2 -> base.copy(fontSize = (fontSizeSp * 1.25f).sp, lineHeight = (fontSizeSp * 1.75f).sp, fontWeight = FontWeight.SemiBold)
            else -> base.copy(fontSize = (fontSizeSp * 1.1f).sp, fontWeight = FontWeight.SemiBold)
        }
        BlockKind.TableRow, BlockKind.Equation -> base.copy(fontFamily = FontFamily.Monospace, fontSize = (fontSizeSp * 0.9f).sp)
        else -> base
    }
}

/** The block's words with their emphasis, highlights in respective colors, mentions in accent. */
private fun annotate(
    block: TextBlock,
    highlights: List<Highlight>,
    colors: dev.neurokitchen.quire.presentation.designsystem.QuireColors,
    finds: List<TextMatch> = emptyList(),
    currentFind: TextMatch? = null,
): AnnotatedString = buildAnnotatedString {
    val prefix = if (block.kind == BlockKind.ListItem) "• " else ""
    append(prefix)
    val wordRanges = mutableListOf<Pair<Int, Int>>()
    block.words.forEachIndexed { i, word ->
        if (i > 0) append(' ')
        val start = length
        val style = SpanStyle(
            fontWeight = if (word.bold) FontWeight.Bold else null,
            fontStyle = if (word.italic) FontStyle.Italic else null,
            color = if (word.mention) colors.accent else Color.Unspecified,
        )
        pushStyle(style)
        append(word.text)
        pop()
        wordRanges.add(start to length)
    }
    val text = prefix + block.words.joinToString(" ") { it.text }
    for (highlight in highlights) {
        val hlColor = when (highlight.color.lowercase()) {
            "green" -> colors.highlightGreen
            "blue" -> colors.highlightBlue
            "purple" -> colors.highlightPurple
            "red" -> colors.highlightRed
            else -> colors.highlightYellow
        }
        val cleanQuote = highlight.quote.replace(Regex("\\s+"), " ").trim()
        if (cleanQuote.isEmpty()) continue

        // 1. Exact or whitespace-normalized substring matching
        var from = text.indexOf(cleanQuote, ignoreCase = true)
        if (from >= 0) {
            while (from >= 0) {
                addStyle(SpanStyle(background = hlColor.copy(alpha = 0.45f)), from, from + cleanQuote.length)
                from = text.indexOf(cleanQuote, from + cleanQuote.length, ignoreCase = true)
            }
        } else {
            // 2. Tokenized word sequence matching (resilient to hyphenation and PDF text extraction line breaks)
            val quoteTokens = cleanQuote.split(" ").map { it.trim().trim { c -> !c.isLetterOrDigit() }.lowercase() }.filter { it.isNotEmpty() }
            if (quoteTokens.isNotEmpty() && block.words.isNotEmpty() && block.words.size >= quoteTokens.size) {
                val blockTokens = block.words.map { it.text.trim().trim { c -> !c.isLetterOrDigit() }.lowercase() }
                for (i in 0..blockTokens.size - quoteTokens.size) {
                    var matches = true
                    for (j in quoteTokens.indices) {
                        if (blockTokens[i + j] != quoteTokens[j]) {
                            matches = false
                            break
                        }
                    }
                    if (matches) {
                        val startPos = wordRanges[i].first
                        val endPos = wordRanges[i + quoteTokens.size - 1].second
                        addStyle(SpanStyle(background = hlColor.copy(alpha = 0.45f)), startPos, endPos)
                    }
                }
            }
        }
    }
    // Find matches: offsets in the block's text, after the list marker.
    for (find in finds) {
        val alpha = if (find == currentFind) 0.6f else 0.3f
        addStyle(SpanStyle(background = colors.findMatch.copy(alpha = alpha)), prefix.length + find.start, prefix.length + find.end)
    }
}

@Composable
private fun Figure(url: String, loadFigure: suspend (String) -> String?) {
    val image: ImageBitmap? by produceState<ImageBitmap?>(null, url) { value = loadFigure(url)?.let(::loadImageFile) }
    image?.let {
        Image(it, contentDescription = null, modifier = Modifier.fillMaxWidth().heightIn(max = 480.dp).padding(vertical = Dimens.SpaceXs))
    }
}

@Composable
private fun TabChip(text: String, selected: Boolean, onClick: () -> Unit) {
    val colors = QuireTheme.colors
    Text(
        text,
        style = MaterialTheme.typography.bodyMedium,
        color = if (selected) colors.onAccent else colors.fillPrimary,
        modifier = Modifier
            .clip(RoundedCornerShape(Dimens.RadiusLarge))
            .background(if (selected) colors.accent else colors.fillQuinary)
            .clickable(onClick = onClick)
            .padding(horizontal = Dimens.SpaceXs, vertical = Dimens.SpaceMin),
    )
}

private val PaperTabKind.label: StringResource
    get() = when (this) {
        PaperTabKind.Abstract -> Res.string.tab_abstract
        PaperTabKind.Paper -> Res.string.tab_paper
        PaperTabKind.Front -> Res.string.tab_front
        PaperTabKind.Figures -> Res.string.tab_figures
        PaperTabKind.Tables -> Res.string.tab_tables
        PaperTabKind.Equations -> Res.string.tab_equations
        PaperTabKind.References -> Res.string.tab_references
    }


/** A list item of the reader: one block, or the table rows from [first] to [last] as one table. */
internal data class ReaderItem(val first: Int, val last: Int = first)

/** The blocks as list items: each run of table rows becomes one item. */
internal fun readerItems(blocks: List<TextBlock>): List<ReaderItem> {
    val items = mutableListOf<ReaderItem>()
    var i = 0
    while (i < blocks.size) {
        var end = i
        if (blocks[i].kind == BlockKind.TableRow) {
            while (end + 1 < blocks.size && blocks[end + 1].kind == BlockKind.TableRow) end++
        }
        items += ReaderItem(i, end)
        i = end + 1
    }
    return items
}

/** For each block, the list item showing it. */
internal fun itemOfBlock(items: List<ReaderItem>, blocks: Int): IntArray {
    val out = IntArray(blocks)
    items.forEachIndexed { index, item -> for (b in item.first..item.last) out[b] = index }
    return out
}
