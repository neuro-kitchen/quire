package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.min
import dev.neurokitchen.quire.domain.model.PageImage
import dev.neurokitchen.quire.domain.model.PageSize
import dev.neurokitchen.quire.domain.model.QuoteLocation
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mapping.toImageBitmap
import kotlin.math.roundToInt

/**
 * The PDF's pages one under the other, rendered as they scroll into view at the pane's width ×
 * `zoom`, with highlights drawn over them. Brings `focused`'s page into view.
 */
@Composable
fun PdfPagesView(
    pages: List<PageSize>,
    zoom: Float,
    locations: List<QuoteLocation>,
    focused: String?,
    loadPage: suspend (index: Int, widthPx: Int) -> PageImage?,
    modifier: Modifier = Modifier,
) {
    val list = rememberLazyListState()
    val focusedPage = locations.firstOrNull { it.highlightId == focused }?.page
    LaunchedEffect(focusedPage) { if (focusedPage != null) list.animateScrollToItem(focusedPage) }
    BoxWithConstraints(modifier.fillMaxSize().background(QuireTheme.colors.sidepane)) {
        val pageWidth = min(maxWidth - Dimens.SpaceLg * 2, MaxPageWidth) * zoom
        LazyColumn(
            state = list,
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
            contentPadding = PaddingValues(Dimens.SpaceMd),
            modifier = Modifier.fillMaxSize(),
        ) {
            itemsIndexed(pages) { index, size ->
                Page(index, size, pageWidth, locations.filter { it.page == index }, focused, loadPage)
            }
        }
    }
}

private val MaxPageWidth = 900.dp

@Composable
private fun Page(
    index: Int,
    size: PageSize,
    width: Dp,
    locations: List<QuoteLocation>,
    focused: String?,
    loadPage: suspend (Int, Int) -> PageImage?,
) {
    val widthPx = with(LocalDensity.current) { width.toPx() }.roundToInt()
    val image: ImageBitmap? by produceState<ImageBitmap?>(null, index, widthPx) {
        value = loadPage(index, widthPx)?.toImageBitmap()
    }
    Box(Modifier.width(width).aspectRatio(size.width / size.height).background(Color.White)) {
        image?.let { Image(it, contentDescription = null, contentScale = ContentScale.FillBounds, modifier = Modifier.fillMaxSize()) }
        Canvas(Modifier.fillMaxSize()) {
            val scale = this.size.width / size.width
            for (location in locations) {
                val alpha = if (location.highlightId == focused) 0.55f else 0.35f
                for (r in location.rects) {
                    drawRect(
                        color = HighlightYellow.copy(alpha = alpha),
                        topLeft = Offset(r.left * scale, r.top * scale),
                        size = Size((r.right - r.left) * scale, (r.bottom - r.top) * scale),
                    )
                }
            }
        }
    }
}
