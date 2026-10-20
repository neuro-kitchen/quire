package dev.neurokitchen.quire.data.repository

import dev.neurokitchen.quire.domain.model.Creator
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.Project
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.model.SortColumn
import dev.neurokitchen.quire.domain.repository.LibraryDescription
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map

/**
 * A fixed in-memory library: for tests, previews, and platforms without the engine bridge yet.
 * Its filtering and sorting stand in for the engine's and are not meant to grow.
 */
class SampleLibraryRepository(
    references: List<Reference> = REFERENCES,
    projects: List<Project> = PROJECTS,
) : LibraryRepository {
    private val references = MutableStateFlow(references)
    private val projects = MutableStateFlow(projects)

    override suspend fun describe() = LibraryDescription("sample", "", "")

    override fun references(query: LibraryQuery): Flow<List<Reference>> = references.map { all ->
        val words = query.text.lowercase().split(' ').filter { it.isNotBlank() }
        all.filter { reference ->
            val inSource = when (val source = query.source) {
                LibrarySource.All -> true
                LibrarySource.Unfiled -> reference.projects.isEmpty()
                LibrarySource.Trash -> false
                is LibrarySource.InProject -> source.path in reference.projects
            }
            val text = listOf(
                reference.citeKey, reference.title, reference.year?.toString().orEmpty(),
                reference.creators.joinToString(" ") { it.displayName },
                reference.tags.joinToString(" "), reference.abstract,
            ).joinToString(" ").lowercase()
            inSource && words.all { it in text } && reference.tags.containsAll(query.tags)
        }.sortedWith(comparator(query.sort, query.ascending))
    }

    override fun projects(): Flow<List<Project>> = projects

    override suspend fun reopen() = Unit

    override suspend fun addReference(reference: Reference) {
        references.value = listOf(reference) + references.value
    }

    override suspend fun updateReference(reference: Reference) {
        references.value = references.value.map { if (it.id == reference.id) reference else it }
    }

    override suspend fun deleteReference(id: String) {
        references.value = references.value.filter { it.id != id }
    }

    override suspend fun createProject(name: String, parentPath: String?) {
        val path = if (parentPath != null) "$parentPath/${name.lowercase().replace(' ', '-')}" else name.lowercase().replace(' ', '-')
        projects.value += Project(path, name, parentPath = parentPath, referenceCount = 0)
    }

    override suspend fun deleteProject(path: String) {
        projects.value = projects.value.filter { it.path != path && it.parentPath != path }
    }

    private fun comparator(column: SortColumn, ascending: Boolean): Comparator<Reference> {
        val byColumn: Comparator<Reference> = when (column) {
            SortColumn.Title -> compareBy { it.title.lowercase() }
            SortColumn.Creator -> compareBy { it.creators.firstOrNull()?.sortName?.lowercase().orEmpty() }
            SortColumn.Year -> compareBy { it.year ?: Int.MAX_VALUE }
            SortColumn.DateAdded -> compareBy { it.dateAdded }
        }
        return if (ascending) byColumn else byColumn.reversed()
    }

    companion object {
        val PROJECTS = listOf(
            Project("thesis", "Thesis", referenceCount = 3),
            Project("thesis/chapter-2", "Chapter 2 – Navigation", parentPath = "thesis", referenceCount = 2),
            Project("deep-learning", "Deep learning", referenceCount = 3),
            Project("reading-group", "Reading group", referenceCount = 2),
        )

        // Real papers with their published metadata; the abstracts are short summaries, not the originals.
        val REFERENCES = listOf(
            Reference(
                "1", "hodgkin1952", "article-journal",
                "A quantitative description of membrane current and its application to conduction and excitation in nerve",
                listOf(person("Hodgkin", "A. L."), person("Huxley", "A. F.")), 1952,
                containerTitle = "The Journal of Physiology", volume = "117", issue = "4", pages = "500–544",
                doi = "10.1113/jphysiol.1952.sp004764",
                abstract = "Sample summary: a mathematical model of the sodium and potassium currents that produce the action potential in the squid giant axon.",
                tags = listOf("neurophysiology", "modelling"), projects = listOf("thesis"), hasPdf = true, reading = ReadingStatus.Read, dateAdded = "2026-09-02",
            ),
            Reference(
                "2", "hubel1962", "article-journal",
                "Receptive fields, binocular interaction and functional architecture in the cat's visual cortex",
                listOf(person("Hubel", "D. H."), person("Wiesel", "T. N.")), 1962,
                containerTitle = "The Journal of Physiology", volume = "160", issue = "1", pages = "106–154",
                doi = "10.1113/jphysiol.1962.sp006837",
                abstract = "Sample summary: single-cell recordings showing simple and complex cells and the columnar organisation of the visual cortex.",
                tags = listOf("neurophysiology", "vision"), projects = listOf("thesis"), hasPdf = true, dateAdded = "2026-09-02",
            ),
            Reference(
                "3", "okeefe1971", "article-journal",
                "The hippocampus as a spatial map. Preliminary evidence from unit activity in the freely-moving rat",
                listOf(person("O'Keefe", "J."), person("Dostrovsky", "J.")), 1971,
                containerTitle = "Brain Research", volume = "34", issue = "1", pages = "171–175",
                doi = "10.1016/0006-8993(71)90358-1",
                abstract = "Sample summary: hippocampal units that fire only when the rat is in a particular place — the first place cells.",
                tags = listOf("hippocampus", "navigation"), projects = listOf("thesis/chapter-2"), hasPdf = true, dateAdded = "2026-09-14",
            ),
            Reference(
                "4", "hafting2005", "article-journal",
                "Microstructure of a spatial map in the entorhinal cortex",
                listOf(person("Hafting", "T."), person("Fyhn", "M."), person("Molden", "S."), person("Moser", "M.-B."), person("Moser", "E. I.")), 2005,
                containerTitle = "Nature", volume = "436", issue = "7052", pages = "801–806",
                doi = "10.1038/nature03721",
                abstract = "Sample summary: entorhinal neurons whose firing fields form a triangular grid across the environment — grid cells.",
                tags = listOf("grid cells", "navigation"), projects = listOf("thesis/chapter-2"), dateAdded = "2026-09-14",
            ),
            Reference(
                "5", "hopfield1982", "article-journal",
                "Neural networks and physical systems with emergent collective computational abilities",
                listOf(person("Hopfield", "J. J.")), 1982,
                containerTitle = "Proceedings of the National Academy of Sciences", volume = "79", issue = "8", pages = "2554–2558",
                doi = "10.1073/pnas.79.8.2554",
                abstract = "Sample summary: a network of simple binary units with symmetric weights that stores memories as stable states.",
                tags = listOf("memory", "modelling"), projects = listOf("reading-group"), hasPdf = true, dateAdded = "2026-10-01",
            ),
            Reference(
                "6", "lecun2015", "article-journal", "Deep learning",
                listOf(person("LeCun", "Y."), person("Bengio", "Y."), person("Hinton", "G.")), 2015,
                containerTitle = "Nature", volume = "521", issue = "7553", pages = "436–444",
                doi = "10.1038/nature14539",
                abstract = "Sample summary: a review of representation learning with many-layered networks and backpropagation.",
                tags = listOf("deep learning", "review"), projects = listOf("deep-learning"), hasPdf = true, dateAdded = "2026-10-05",
            ),
            Reference(
                "7", "vaswani2017", "paper-conference", "Attention is all you need",
                listOf(person("Vaswani", "A."), person("Shazeer", "N."), person("Parmar", "N."), person("Uszkoreit", "J."), person("Jones", "L."), person("Gomez", "A. N."), person("Kaiser", "Ł."), person("Polosukhin", "I.")), 2017,
                containerTitle = "Advances in Neural Information Processing Systems 30", pages = "5998–6008",
                abstract = "Sample summary: the Transformer, a sequence model built only from attention, without recurrence or convolution.",
                tags = listOf("deep learning", "transformers"), projects = listOf("deep-learning", "reading-group"), hasPdf = true, dateAdded = "2026-10-05",
            ),
            Reference(
                "8", "he2016", "paper-conference", "Deep residual learning for image recognition",
                listOf(person("He", "K."), person("Zhang", "X."), person("Ren", "S."), person("Sun", "J.")), 2016,
                containerTitle = "IEEE Conference on Computer Vision and Pattern Recognition (CVPR)", pages = "770–778",
                doi = "10.1109/CVPR.2016.90",
                abstract = "Sample summary: residual connections that make very deep convolutional networks trainable.",
                tags = listOf("deep learning", "vision"), projects = listOf("deep-learning"), dateAdded = "2026-10-06",
            ),
            Reference(
                "9", "kandel2013", "book", "Principles of Neural Science",
                listOf(person("Kandel", "E. R."), person("Schwartz", "J. H."), person("Jessell", "T. M."), person("Siegelbaum", "S. A."), person("Hudspeth", "A. J.")), 2013,
                publisher = "McGraw-Hill",
                tags = listOf("textbook"), projects = listOf("thesis"), dateAdded = "2026-08-28",
            ),
            Reference(
                "10", "buzsaki2006", "book", "Rhythms of the Brain",
                listOf(person("Buzsáki", "G.")), 2006,
                publisher = "Oxford University Press",
                doi = "10.1093/acprof:oso/9780195301069.001.0001",
                tags = listOf("oscillations", "textbook"), dateAdded = "2026-10-18",
            ),
        )

        val EXTENDED_REFERENCES = listOf(
            Reference(
                "11", "jumper2021", "article-journal",
                "Highly accurate protein structure prediction with AlphaFold",
                listOf(person("Jumper", "J."), person("Evans", "R."), person("Pritzel", "A."), person("Green", "T."), person("Figurnov", "M."), person("Hassabis", "D.")), 2021,
                containerTitle = "Nature", volume = "596", issue = "7873", pages = "583–589",
                doi = "10.1038/s41586-021-03819-2",
                abstract = "Predicting a protein's 3D structure from its amino acid sequence using neural networks with evolutionary constraints.",
                tags = listOf("structural-biology", "deep learning", "proteins"), projects = listOf("deep-learning", "reading-group"), hasPdf = true, reading = ReadingStatus.Read, dateAdded = "2026-10-18",
            ),
            Reference(
                "12", "doudna2014", "article-journal",
                "The new frontier of genome engineering with CRISPR-Cas9",
                listOf(person("Doudna", "J. A."), person("Charpentier", "E.")), 2014,
                containerTitle = "Science", volume = "346", issue = "6213", pages = "1258096",
                doi = "10.1126/science.1258096",
                abstract = "A review of the RNA-guided CRISPR-Cas9 system for site-specific genomic engineering across cellular organisms.",
                tags = listOf("genetics", "crispr", "molecular-biology"), projects = listOf("thesis"), hasPdf = true, reading = ReadingStatus.Reading, dateAdded = "2026-10-19",
            ),
            Reference(
                "13", "ho2020", "paper-conference",
                "Denoising Diffusion Probabilistic Models",
                listOf(person("Ho", "J."), person("Jain", "A."), person("Abbeel", "P.")), 2020,
                containerTitle = "Advances in Neural Information Processing Systems 33", pages = "6840–6851",
                abstract = "High quality image synthesis results using diffusion probabilistic models with non-equilibrium thermodynamics connections.",
                tags = listOf("diffusion", "generative-ai", "deep learning"), projects = listOf("deep-learning"), hasPdf = true, dateAdded = "2026-10-19",
            ),
            Reference(
                "14", "dao2022", "paper-conference",
                "FlashAttention: Fast and Memory-Efficient Exact Attention with IO-Awareness",
                listOf(person("Dao", "T."), person("Fu", "D. Y."), person("Ermon", "S."), person("Rudra", "A."), person("Ré", "C.")), 2022,
                containerTitle = "Advances in Neural Information Processing Systems 35", pages = "16344–16359",
                doi = "10.48550/arXiv.2205.14135",
                abstract = "An exact attention algorithm that reduces memory reads/writes between GPU SRAM and HBM to accelerate Transformers.",
                tags = listOf("transformers", "gpu", "systems"), projects = listOf("deep-learning"), hasPdf = true, reading = ReadingStatus.Reading, dateAdded = "2026-10-20",
            ),
            Reference(
                "15", "shannon1948", "article-journal",
                "A Mathematical Theory of Communication",
                listOf(person("Shannon", "C. E.")), 1948,
                containerTitle = "The Bell System Technical Journal", volume = "27", issue = "3", pages = "379–423",
                doi = "10.1002/j.1538-7305.1948.tb01338.x",
                abstract = "The foundation of information theory: entropy, channel capacity, and noiseless coding theorems.",
                tags = listOf("information-theory", "mathematics", "classic"), projects = listOf("thesis"), hasPdf = true, reading = ReadingStatus.Read, dateAdded = "2026-10-20",
            ),
        )
    }
}

private fun person(family: String, given: String) = Creator(sortName = family, displayName = "$family, $given")
