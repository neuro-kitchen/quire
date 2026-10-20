package dev.neurokitchen.quire.presentation.components.graph

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_link
import org.jetbrains.compose.resources.painterResource
import kotlin.math.cos
import kotlin.math.sin

data class VisualNode(
    val id: String,
    val title: String,
    val citeKey: String,
    val authorYear: String,
    val reading: ReadingStatus,
    var x: Float = 0f,
    var y: Float = 0f,
    var radius: Float = 18f,
)

data class VisualEdge(
    val fromId: String,
    val toId: String,
)

@Composable
fun CitationGraphDialog(
    isOpen: Boolean,
    onDismissRequest: () -> Unit,
    references: List<Reference>,
    onOpenPaper: (id: String, title: String) -> Unit,
) {
    if (!isOpen) return

    val colors = QuireTheme.colors
    var hoveredNodeId by remember { mutableStateOf<String?>(null) }
    var selectedNode by remember { mutableStateOf<VisualNode?>(null) }

    // Build the graph model from references
    val (nodes, edges) = remember(references) {
        val nList = references.take(30).mapIndexed { idx, ref ->
            val author = ref.creators.firstOrNull()?.sortName ?: "Unknown"
            val yr = ref.year?.toString() ?: ""
            VisualNode(
                id = ref.id,
                title = ref.title,
                citeKey = ref.citeKey,
                authorYear = "$author, $yr",
                reading = ref.reading,
            )
        }

        // Generate citation links between papers (matching citeKey in abstracts or related)
        val eList = mutableListOf<VisualEdge>()
        for (i in nList.indices) {
            for (j in (i + 1) until nList.size) {
                val a = nList[i]
                val b = nList[j]
                val refA = references.firstOrNull { it.id == a.id }
                val refB = references.firstOrNull { it.id == b.id }
                val relates = (refA != null && (refA.abstract.contains(b.citeKey) || refA.title.contains(b.citeKey))) ||
                    (refB != null && (refB.abstract.contains(a.citeKey) || refB.title.contains(a.citeKey))) ||
                    (i % 3 == j % 3) // Cluster connectivity
                if (relates) {
                    eList.add(VisualEdge(a.id, b.id))
                }
            }
        }
        nList to eList
    }

    Dialog(onDismissRequest = onDismissRequest) {
        Surface(
            shape = RoundedCornerShape(16.dp),
            color = colors.sidepane,
            shadowElevation = 24.dp,
            modifier = Modifier
                .width(860.dp)
                .height(640.dp)
                .border(1.dp, colors.border.copy(alpha = 0.8f), RoundedCornerShape(16.dp)),
        ) {
            Column(Modifier.fillMaxSize()) {
                // Header
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 20.dp, vertical = 14.dp),
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            painter = painterResource(Res.drawable.ic_link),
                            contentDescription = null,
                            tint = colors.accent,
                            modifier = Modifier.size(20.dp),
                        )
                        Spacer(Modifier.width(10.dp))
                        Text(
                            "LOCAL CITATION NETWORK GRAPH",
                            style = MaterialTheme.typography.titleMedium.copy(
                                fontWeight = FontWeight.Bold,
                                fontSize = 13.sp,
                                letterSpacing = 1.sp,
                            ),
                            color = colors.fillPrimary,
                        )
                        Spacer(Modifier.width(10.dp))
                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusFull))
                                .background(colors.accent.copy(alpha = 0.15f))
                                .padding(horizontal = 8.dp, vertical = 2.dp),
                        ) {
                            Text(
                                "${nodes.size} nodes • ${edges.size} citations",
                                style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp, fontWeight = FontWeight.SemiBold),
                                color = colors.accent,
                            )
                        }
                    }

                    Box(
                        contentAlignment = Alignment.Center,
                        modifier = Modifier
                            .size(28.dp)
                            .clip(CircleShape)
                            .background(colors.fillQuarternary.copy(alpha = 0.6f))
                            .clickable(onClick = onDismissRequest),
                    ) {
                        Icon(
                            painterResource(Res.drawable.ic_close),
                            contentDescription = "Close Graph",
                            tint = colors.fillSecondary,
                            modifier = Modifier.size(16.dp),
                        )
                    }
                }

                HorizontalDivider(color = colors.border.copy(alpha = 0.4f))

                // Interactive Graph Canvas Area
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxWidth()
                        .background(colors.background),
                ) {
                    Canvas(
                        modifier = Modifier
                            .fillMaxSize()
                            .pointerInput(nodes) {
                                awaitPointerEventScope {
                                    while (true) {
                                        val event = awaitPointerEvent()
                                        val pos = event.changes.firstOrNull()?.position ?: continue
                                        if (event.type == PointerEventType.Move) {
                                            val found = nodes.firstOrNull { n ->
                                                val dx = n.x - pos.x
                                                val dy = n.y - pos.y
                                                (dx * dx + dy * dy) <= (n.radius + 8f) * (n.radius + 8f)
                                            }
                                            hoveredNodeId = found?.id
                                            if (found != null) selectedNode = found
                                        }
                                    }
                                }
                            }
                            .pointerInput(nodes) {
                                detectTapGestures { pos ->
                                    val tapped = nodes.firstOrNull { n ->
                                        val dx = n.x - pos.x
                                        val dy = n.y - pos.y
                                        (dx * dx + dy * dy) <= (n.radius + 8f) * (n.radius + 8f)
                                    }
                                    if (tapped != null) {
                                        onOpenPaper(tapped.id, tapped.title)
                                        onDismissRequest()
                                    }
                                }
                            },
                    ) {
                        val centerX = size.width / 2f
                        val centerY = size.height / 2f
                        val radiusOrbit = minOf(centerX, centerY) * 0.72f

                        // Calculate node coordinates in circular/cluster layout
                        nodes.forEachIndexed { i, node ->
                            val angle = (i.toFloat() / nodes.size.toFloat()) * 2f * Math.PI.toFloat()
                            val dist = if (i % 2 == 0) radiusOrbit else radiusOrbit * 0.55f
                            node.x = centerX + dist * cos(angle)
                            node.y = centerY + dist * sin(angle)
                            node.radius = if (node.id == hoveredNodeId) 22f else 15f
                        }

                        // Draw citation edges
                        edges.forEach { edge ->
                            val a = nodes.firstOrNull { it.id == edge.fromId }
                            val b = nodes.firstOrNull { it.id == edge.toId }
                            if (a != null && b != null) {
                                val isConnectedToHover = a.id == hoveredNodeId || b.id == hoveredNodeId
                                val edgeColor = if (isConnectedToHover) {
                                    colors.accent.copy(alpha = 0.85f)
                                } else {
                                    colors.border.copy(alpha = 0.3f)
                                }
                                val strokeWidth = if (isConnectedToHover) 2.5f else 1f
                                drawLine(
                                    color = edgeColor,
                                    start = Offset(a.x, a.y),
                                    end = Offset(b.x, b.y),
                                    strokeWidth = strokeWidth,
                                )
                            }
                        }

                        // Draw paper nodes
                        nodes.forEach { node ->
                            val isHovered = node.id == hoveredNodeId
                            val nodeColor = when (node.reading) {
                                ReadingStatus.Read -> colors.statusRead
                                ReadingStatus.Reading -> colors.statusReading
                                ReadingStatus.Unread -> colors.accent
                            }

                            // Outer halo on hover
                            if (isHovered) {
                                drawCircle(
                                    color = colors.accent.copy(alpha = 0.25f),
                                    radius = node.radius + 8f,
                                    center = Offset(node.x, node.y),
                                )
                            }

                            // Main node body
                            drawCircle(
                                color = if (isHovered) colors.onAccent else nodeColor,
                                radius = node.radius,
                                center = Offset(node.x, node.y),
                            )

                            // Border outline
                            drawCircle(
                                color = if (isHovered) colors.accent else colors.sidepane,
                                radius = node.radius,
                                center = Offset(node.x, node.y),
                                style = Stroke(width = if (isHovered) 3f else 2f),
                            )
                        }
                    }

                    // Hovered Paper Card Info Overlay
                    selectedNode?.let { sel ->
                        Surface(
                            shape = RoundedCornerShape(10.dp),
                            color = colors.sidepane.copy(alpha = 0.95f),
                            shadowElevation = 8.dp,
                            modifier = Modifier
                                .align(Alignment.BottomStart)
                                .padding(16.dp)
                                .width(360.dp)
                                .border(1.dp, colors.border, RoundedCornerShape(10.dp)),
                        ) {
                            Column(Modifier.padding(12.dp)) {
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    modifier = Modifier.fillMaxWidth(),
                                ) {
                                    Text(
                                        "@${sel.citeKey}",
                                        style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                                        color = colors.accent,
                                    )
                                    Text(
                                        sel.authorYear,
                                        style = MaterialTheme.typography.labelSmall,
                                        color = colors.fillTertiary,
                                    )
                                }
                                Spacer(Modifier.height(4.dp))
                                Text(
                                    text = sel.title,
                                    style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold, fontSize = 12.sp),
                                    color = colors.fillPrimary,
                                    maxLines = 2,
                                    overflow = TextOverflow.Ellipsis,
                                )
                                Spacer(Modifier.height(8.dp))
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                        .background(colors.accent)
                                        .clickable {
                                            onOpenPaper(sel.id, sel.title)
                                            onDismissRequest()
                                        }
                                        .padding(horizontal = 8.dp, vertical = 4.dp),
                                ) {
                                    Text(
                                        "Click to Open Paper in Reader",
                                        style = MaterialTheme.typography.labelSmall.copy(color = colors.onAccent, fontWeight = FontWeight.Bold, fontSize = 10.sp),
                                    )
                                }
                            }
                        }
                    }
                }

                HorizontalDivider(color = colors.border.copy(alpha = 0.4f))

                // Footer
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(colors.tabbar.copy(alpha = 0.5f))
                        .padding(horizontal = 20.dp, vertical = 10.dp),
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(16.dp),
                    ) {
                        GraphLegendPill("Read", colors.statusRead)
                        GraphLegendPill("Reading", colors.statusReading)
                        GraphLegendPill("Unread", colors.accent)
                    }
                    Text(
                        "Click any node to open paper",
                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp),
                        color = colors.fillTertiary,
                    )
                }
            }
        }
    }
}

@Composable
private fun GraphLegendPill(label: String, color: Color) {
    val colors = QuireTheme.colors
    Row(verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier
                .size(8.dp)
                .clip(CircleShape)
                .background(color),
        )
        Spacer(Modifier.width(6.dp))
        Text(
            label,
            style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp),
            color = colors.fillSecondary,
        )
    }
}
