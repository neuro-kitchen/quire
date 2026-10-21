package dev.neurokitchen.quire.presentation.screens.reader.components

import dev.neurokitchen.quire.presentation.screens.reader.TextMatch
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.lazy.itemsIndexed
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
) {
    val tab = tabs.getOrNull(selectedTab) ?: return
    val list = rememberLazyListState()
    val focusedQuote = highlights.firstOrNull { it.id == focused }?.quote
    val themeColors = readerColors(readerTheme)
    var localSelection by remember(selectedTab) { mutableStateOf<Pair<String, String>?>(null) }

    val current = findMatches.getOrNull(currentFind)
    LaunchedEffect(current) { if (current != null) list.animateScrollToItem(current.block) }

    LaunchedEffect(focusedQuote, selectedTab) {
        val index = focusedQuote?.let { q ->
            val clean = q.replace(Regex("\\s+"), " ").trim().lowercase()
            tab.blocks.indexOfFirst { block ->
                val blockText = block.text.replace(Regex("\\s+"), " ").lowercase()
                clean in blockText || blockText in clean
            }
        } ?: -1
        if (index >= 0) list.animateScrollToItem(index)
    }

    Column(modifier.fillMaxSize().background(themeColors.background)) {
        Row(
            Modifier
                .fillMaxWidth()
                .horizontalScroll(rememberScrollState())
                .background(themeColors.surface)
                .padding(Dimens.SpaceXs),
            horizontalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            tabs.forEachIndexed { index, t ->
                val label = stringResource(t.kind.label) + (t.count?.let { " ($it)" } ?: "")
                TabChip(label, selected = index == selectedTab) { onTabClick(index) }
            }
        }

        // Reading Progress Bar at the top of the Text viewport
        val textProgress by remember(tab, list) {
            derivedStateOf {
                if (tab.blocks.isNotEmpty()) {
                    ((list.firstVisibleItemIndex + 1).toFloat() / tab.blocks.size.toFloat()).coerceIn(0f, 1f)
                } else 0f
            }
        }
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(3.dp)
                .background(QuireTheme.colors.border.copy(alpha = 0.25f)),
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth(textProgress)
                    .fillMaxHeight()
                    .background(QuireTheme.colors.accent),
            )
        }
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.TopCenter) {
            LazyColumn(
                state = list,
                verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
                contentPadding = PaddingValues(vertical = Dimens.SpaceMd, horizontal = Dimens.SpaceLg),
                modifier = Modifier.widthIn(max = ReadingWidth).fillMaxSize(),
            ) {
                itemsIndexed(tab.blocks) { blockIndex, block ->
                    when (val kind = block.kind) {
                        is BlockKind.Image -> Figure(kind.url, loadFigure)
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
                        onSelection(null)
                    },
                    onAddNote = {
                        onAddNote(sel.first)
                        localSelection = null
                        onSelection(null)
                    },
                    onCopy = {
                        localSelection = null
                        onSelection(null)
                    },
                    onCopyCitation = {
                        localSelection = null
                        onSelection(null)
                    },
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .padding(bottom = Dimens.SpaceLg),
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
            .padding(horizontal = Dimens.SpaceXs, vertical = 4.dp),
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

