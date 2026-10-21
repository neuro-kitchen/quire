package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.SubcomposeLayout
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens

/**
 * A paper's table: a column per cell, each as wide as its widest cell (up to [maxColumnWidth], then
 * wrapped), the first row as the header, numbers aligned right, rows striped. Wider than the space
 * it has, it scrolls sideways inside itself. The text can be selected and copied.
 */
@Composable
fun PaperTable(
    rows: List<List<String>>,
    textStyle: TextStyle,
    textColor: Color,
    lineColor: Color,
    stripeColor: Color,
    modifier: Modifier = Modifier,
    maxColumnWidth: Dp = 320.dp,
) {
    if (rows.isEmpty()) return
    val columns = rows.maxOf { it.size }
    val shape = RoundedCornerShape(Dimens.RadiusBase)
    SelectionContainer {
        Box(
            modifier
                .border(1.dp, lineColor, shape)
                .horizontalScroll(rememberScrollState()),
        ) {
            TableGrid(rows, columns, maxColumnWidth) { row, column ->
                val text = rows[row].getOrElse(column) { "" }
                val header = row == 0 && rows.size > 1
                Box(
                    Modifier
                        .background(if (!header && row % 2 == 0) stripeColor else Color.Transparent)
                        .padding(horizontal = Dimens.SpaceSm, vertical = Dimens.SpaceXs),
                ) {
                    Text(
                        text,
                        style = textStyle.copy(
                            color = textColor,
                            fontWeight = if (header) FontWeight.SemiBold else textStyle.fontWeight,
                            textAlign = if (!header && isNumber(text)) TextAlign.End else TextAlign.Start,
                        ),
                        // The cell's whole width, so a number can sit at its right edge.
                        modifier = Modifier.fillMaxWidth(),
                    )
                }
            }
        }
    }
}

/**
 * Lays the cells out in a grid: each column as wide as its widest cell's natural width (up to the
 * limit, then wrapped), each row as tall as its tallest cell at those widths; every cell is placed
 * at exactly its column's width and row's height, so alignment and stripes line up.
 */
@Composable
private fun TableGrid(rows: List<List<String>>, columns: Int, maxColumnWidth: Dp, cell: @Composable (row: Int, column: Int) -> Unit) {
    SubcomposeLayout { _ ->
        val maxWidth = maxColumnWidth.roundToPx()
        val cells = rows.indices.flatMap { r -> (0 until columns).map { c -> r to c } }
        // Widths from each cell's natural (intrinsic) width; cells fill their width, so measuring
        // them would report the limit.
        val measuring = cells.associateWith { (r, c) -> subcompose("measure-$r-$c") { cell(r, c) }.first() }
        val widths = IntArray(columns) { c ->
            rows.indices.maxOf { r -> measuring.getValue(r to c).maxIntrinsicWidth(Constraints.Infinity).coerceAtMost(maxWidth) }
        }
        // Heights at those widths (a long cell wraps).
        val heights = IntArray(rows.size) { r ->
            (0 until columns).maxOf { c -> measuring.getValue(r to c).measure(Constraints.fixedWidth(widths[c])).height }
        }
        // Pass 2: each cell at its column's width and its row's height.
        val placed = cells.associateWith { (r, c) ->
            subcompose("place-$r-$c") { cell(r, c) }.first().measure(Constraints.fixed(widths[c], heights[r]))
        }
        layout(widths.sum(), heights.sum()) {
            var y = 0
            for (r in rows.indices) {
                var x = 0
                for (c in 0 until columns) {
                    placed.getValue(r to c).place(x, y)
                    x += widths[c]
                }
                y += heights[r]
            }
        }
    }
}

/** A number, a percentage or a ± value (`12.5`, `−3`, `45%`, `0.31 ± 0.02`): aligned right. */
private fun isNumber(text: String): Boolean {
    val t = text.trim()
    return t.isNotEmpty() && t.all { it.isDigit() || it in ".,-−+±%() *eE" } && t.any(Char::isDigit)
}
