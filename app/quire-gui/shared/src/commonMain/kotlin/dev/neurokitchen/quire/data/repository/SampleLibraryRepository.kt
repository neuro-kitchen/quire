package dev.neurokitchen.quire.data.repository

import dev.neurokitchen.quire.domain.model.Creator
import dev.neurokitchen.quire.domain.model.AddOutcome
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.Project
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.model.SortColumn
import dev.neurokitchen.quire.domain.model.Tag
import dev.neurokitchen.quire.domain.repository.LibraryDescription
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
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

    override suspend fun addReference(reference: Reference): AddOutcome {
        references.value = listOf(reference) + references.value
        return AddOutcome(isNew = true, reference = reference)
    }

    override suspend fun addByIdentifier(input: String, projectPath: String?): AddOutcome {
        val trimmed = input.trim()
        val isDoi = trimmed.startsWith("10.") || trimmed.contains("doi.org/")
        val cleanDoi = if (trimmed.contains("doi.org/")) trimmed.substringAfter("doi.org/").trim() else trimmed
        val citeKey = if (isDoi) "doi_" + cleanDoi.replace("/", "_").replace(".", "_").takeLast(10) else "ref_${references.value.size + 1}"
        val title = if (isDoi) "Paper (DOI: $cleanDoi)" else "Paper ($trimmed)"
        val newRef = Reference(
            id = (references.value.size + 1).toString(),
            citeKey = citeKey,
            kind = "article-journal",
            title = title,
            creators = listOf(person("Author", "A.")),
            year = 2024,
            doi = if (isDoi) cleanDoi else "",
            projects = if (projectPath != null) listOf(projectPath) else emptyList(),
            hasPdf = false,
            dateAdded = "2026-10-22",
        )
        references.value = listOf(newRef) + references.value
        return AddOutcome(isNew = true, reference = newRef)
    }

    override suspend fun updateReference(reference: Reference) {
        references.value = references.value.map { if (it.id == reference.id) reference else it }
    }

    override suspend fun deleteReference(id: String) {
        references.value = references.value.filter { it.id != id }
    }

    override suspend fun createProject(name: String, parentPath: String?): String {
        val path = if (parentPath != null) "$parentPath/${name.lowercase().replace(' ', '-')}" else name.lowercase().replace(' ', '-')
        projects.value += Project(path, name, parentPath = parentPath, referenceCount = 0)
        return path
    }

    override suspend fun addToProject(id: String, projectPath: String): Boolean {
        var added = false
        references.value = references.value.map {
            if (it.id == id && projectPath !in it.projects) it.copy(projects = it.projects + projectPath).also { added = true } else it
        }
        return added
    }

    override suspend fun tag(id: String, names: List<String>) {
        references.value = references.value.map { if (it.id == id) it.copy(tags = (it.tags + names).distinct()) else it }
    }

    override suspend fun untag(id: String, names: List<String>) {
        references.value = references.value.map { if (it.id == id) it.copy(tags = it.tags - names.toSet()) else it }
    }

    override suspend fun deleteProject(path: String) {
        projects.value = projects.value.filter { it.path != path && it.parentPath != path }
    }

    override suspend fun renameProject(path: String, name: String): String {
        val renamed = (path.substringBeforeLast('/', "").let { if (it.isEmpty()) "" else "$it/" }) + name.lowercase().replace(' ', '-')
        fun moved(p: String) = if (p == path || p.startsWith("$path/")) renamed + p.removePrefix(path) else p
        projects.value = projects.value.map {
            it.copy(path = moved(it.path), name = if (it.path == path) name else it.name, parentPath = it.parentPath?.let(::moved))
        }
        references.value = references.value.map { it.copy(projects = it.projects.map(::moved)) }
        return renamed
    }

    /** Tag colours by name; the tags themselves are the references'. */
    private val tagColors = MutableStateFlow(mapOf("deep learning" to "#2EA8E5"))

    override fun tags(): Flow<List<Tag>> = combine(references, tagColors) { all, colors ->
        all.flatMap { it.tags }.groupingBy { it }.eachCount().map { (name, count) -> Tag(name, colors[name], count) }.sortedBy { it.name }
    }

    override suspend fun renameTag(old: String, new: String): Boolean {
        references.value = references.value.map { r -> r.copy(tags = r.tags.map { if (it == old) new else it }.distinct()) }
        tagColors.value[old]?.let { tagColors.value = tagColors.value - old + (new to it) }
        return true
    }

    override suspend fun deleteTag(name: String): Int {
        val count = references.value.count { name in it.tags }
        references.value = references.value.map { it.copy(tags = it.tags - name) }
        tagColors.value -= name
        return count
    }

    override suspend fun colorTag(name: String, color: String?, position: Int?): Tag {
        tagColors.value = if (color == null) tagColors.value - name else tagColors.value + (name to color)
        return Tag(name, color)
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
            Reference(
                "16", "touvron2023", "preprint",
                "LLaMA: Open and Efficient Foundation Language Models",
                listOf(person("Touvron", "H."), person("Lavril", "T."), person("Izacard", "G."), person("Martinet", "X."), person("Lample", "G.")), 2023,
                containerTitle = "arXiv:2302.13971",
                doi = "10.48550/arXiv.2302.13971",
                abstract = "A collection of foundation language models ranging from 7B to 65B parameters trained exclusively on publicly available datasets.",
                tags = listOf("deep-learning", "transformers", "nlp"), projects = listOf("deep-learning"), hasPdf = true, reading = ReadingStatus.Reading, dateAdded = "2026-10-21",
            ),
            Reference(
                "17", "mnih2015", "article-journal",
                "Human-level control through deep reinforcement learning",
                listOf(person("Mnih", "V."), person("Kavukcuoglu", "K."), person("Silver", "D."), person("Hassabis", "D.")), 2015,
                containerTitle = "Nature", volume = "518", issue = "7540", pages = "529–533",
                doi = "10.1038/nature14236",
                abstract = "Reinforcement learning combined with deep neural networks mastering Atari 2600 games directly from raw pixel inputs.",
                tags = listOf("reinforcement-learning", "deep-learning"), projects = listOf("deep-learning"), hasPdf = true, reading = ReadingStatus.Read, dateAdded = "2026-10-21",
            ),
            Reference(
                "18", "silver2016", "article-journal",
                "Mastering the game of Go with deep neural networks and tree search",
                listOf(person("Silver", "D."), person("Huang", "A."), person("Maddison", "C. J."), person("Hassabis", "D.")), 2016,
                containerTitle = "Nature", volume = "529", issue = "7587", pages = "484–489",
                doi = "10.1038/nature16961",
                abstract = "AlphaGo combines value networks to evaluate board positions with policy networks to select moves.",
                tags = listOf("reinforcement-learning", "go", "game-ai"), projects = listOf("deep-learning"), hasPdf = true, reading = ReadingStatus.Read, dateAdded = "2026-10-21",
            ),
            Reference(
                "19", "goodfellow2014", "paper-conference",
                "Generative Adversarial Nets",
                listOf(person("Goodfellow", "I."), person("Pouget-Abadie", "J."), person("Mirza", "M."), person("Xu", "B."), person("Bengio", "Y.")), 2014,
                containerTitle = "Advances in Neural Information Processing Systems 27", pages = "2672–2680",
                abstract = "A framework for estimating generative models via an adversarial process with generator and discriminator networks.",
                tags = listOf("generative-ai", "gan", "deep-learning"), projects = listOf("deep-learning"), hasPdf = true, reading = ReadingStatus.Read, dateAdded = "2026-10-21",
            ),
            Reference(
                "20", "devlin2018", "paper-conference",
                "BERT: Pre-training of Deep Bidirectional Transformers for Language Understanding",
                listOf(person("Devlin", "J."), person("Chang", "M.-W."), person("Lee", "K."), person("Toutanova", "K.")), 2018,
                containerTitle = "Proceedings of NAACL-HLT 2019", pages = "4171–4186",
                doi = "10.18653/v1/N19-1423",
                abstract = "Pre-trained bidirectional representations conditioned on both left and right context in all transformer layers.",
                tags = listOf("nlp", "transformers", "bert"), projects = listOf("deep-learning"), hasPdf = true, reading = ReadingStatus.Read, dateAdded = "2026-10-21",
            ),
            Reference(
                "21", "brown2020", "paper-conference",
                "Language Models are Few-Shot Learners",
                listOf(person("Brown", "T. B."), person("Mann", "B."), person("Ryder", "N."), person("Subbiah", "M."), person("Amodei", "D.")), 2020,
                containerTitle = "Advances in Neural Information Processing Systems 33", pages = "1877–1901",
                abstract = "Scaling language models greatly improves task-agnostic few-shot performance (GPT-3).",
                tags = listOf("nlp", "gpt", "scaling-laws"), projects = listOf("deep-learning", "reading-group"), hasPdf = true, reading = ReadingStatus.Reading, dateAdded = "2026-10-22",
            ),
            Reference(
                "22", "rumelhart1986", "article-journal",
                "Learning representations by back-propagating errors",
                listOf(person("Rumelhart", "D. E."), person("Hinton", "G. E."), person("Williams", "R. J.")), 1986,
                containerTitle = "Nature", volume = "323", issue = "6088", pages = "533–536",
                doi = "10.1038/323533a0",
                abstract = "A procedure for learning representations by adjusting the weights of connectionist networks through backpropagation.",
                tags = listOf("backpropagation", "neural-networks", "classic"), projects = listOf("thesis"), hasPdf = true, reading = ReadingStatus.Read, dateAdded = "2026-10-22",
            ),
            Reference(
                "23", "turing1950", "article-journal",
                "Computing Machinery and Intelligence",
                listOf(person("Turing", "A. M.")), 1950,
                containerTitle = "Mind", volume = "LIX", issue = "236", pages = "433–460",
                doi = "10.1093/mind/LIX.236.433",
                abstract = "The imitation game and philosophical foundations for whether machines can think.",
                tags = listOf("philosophy", "ai", "foundational"), projects = listOf("thesis"), reading = ReadingStatus.Read, dateAdded = "2026-10-22",
            ),
            Reference(
                "24", "moser2008", "article-journal",
                "Place cells, grid cells, and the brain's spatial representation system",
                listOf(person("Moser", "E. I."), person("Kropff", "E."), person("Moser", "M.-B.")), 2008,
                containerTitle = "Annual Review of Neuroscience", volume = "31", pages = "69–89",
                doi = "10.1146/annurev.neuro.31.061307.090723",
                abstract = "A synthesis of how place cells in the hippocampus and grid cells in the medial entorhinal cortex map space.",
                tags = listOf("navigation", "grid cells", "hippocampus"), projects = listOf("thesis/chapter-2"), hasPdf = true, reading = ReadingStatus.Reading, dateAdded = "2026-10-22",
            ),
            Reference(
                "25", "collingridge1983", "article-journal",
                "Excitatory amino acids in synaptic transmission in the Schaffer collateral-commissural pathway of the rat hippocampus",
                listOf(person("Collingridge", "G. L."), person("Kehl", "S. J."), person("McLennan", "H.")), 1983,
                containerTitle = "The Journal of Physiology", volume = "334", issue = "1", pages = "33–46",
                doi = "10.1113/jphysiol.1983.sp014478",
                abstract = "Demonstration of NMDA receptor involvement in the induction of hippocampal long-term potentiation (LTP).",
                tags = listOf("ltp", "synaptic-plasticity", "nmda"), projects = listOf("thesis"), hasPdf = true, reading = ReadingStatus.Unread, dateAdded = "2026-10-22",
            ),
            Reference(
                "26", "bliss1973", "article-journal",
                "Long-lasting potentiation of synaptic transmission in the dentate area of the anaesthetized rabbit",
                listOf(person("Bliss", "T. V. P."), person("Lømo", "T.")), 1973,
                containerTitle = "The Journal of Physiology", volume = "232", issue = "2", pages = "331–356",
                doi = "10.1113/jphysiol.1973.sp010273",
                abstract = "The original discovery of long-term potentiation (LTP), providing the first experimental evidence for Hebbian plasticity.",
                tags = listOf("ltp", "memory", "synaptic-transmission"), projects = listOf("thesis"), hasPdf = true, reading = ReadingStatus.Read, dateAdded = "2026-10-22",
            ),
            Reference(
                "27", "lamport1978", "article-journal",
                "Time, Clocks, and the Ordering of Events in a Distributed System",
                listOf(person("Lamport", "L.")), 1978,
                containerTitle = "Communications of the ACM", volume = "21", issue = "7", pages = "558–565",
                doi = "10.1145/359545.359563",
                abstract = "Logical clocks and partial ordering of events in distributed systems without a shared global clock.",
                tags = listOf("distributed-systems", "clocks", "classic"), projects = listOf("reading-group"), hasPdf = true, reading = ReadingStatus.Read, dateAdded = "2026-10-22",
            ),
            Reference(
                "28", "dean2004", "paper-conference",
                "MapReduce: Simplified Data Processing on Large Clusters",
                listOf(person("Dean", "J."), person("Ghemawat", "S.")), 2004,
                containerTitle = "6th USENIX Symposium on Operating Systems Design and Implementation (OSDI '04)", pages = "137–150",
                abstract = "A programming model and associated implementation for processing and generating large datasets automatically parallelized across clusters.",
                tags = listOf("distributed-systems", "big-data", "systems"), projects = listOf("reading-group"), hasPdf = true, reading = ReadingStatus.Read, dateAdded = "2026-10-22",
            ),
            Reference(
                "29", "ongaro2014", "paper-conference",
                "In Search of an Understandable Consensus Algorithm",
                listOf(person("Ongaro", "D."), person("Ousterhout", "J.")), 2014,
                containerTitle = "2014 USENIX Annual Technical Conference (USENIX ATC 14)", pages = "305–319",
                abstract = "The Raft consensus algorithm, designed to be more understandable than Paxos while providing equivalent safety and fault tolerance.",
                tags = listOf("consensus", "raft", "distributed-systems"), projects = listOf("reading-group"), hasPdf = true, reading = ReadingStatus.Reading, dateAdded = "2026-10-22",
            ),
            Reference(
                "30", "roelfsema2014", "article-journal",
                "Basic mechanisms of cognitive reflection and attentional selection in primate visual cortex",
                listOf(person("Roelfsema", "P. R."), person("Treue", "S.")), 2014,
                containerTitle = "Neuron", volume = "83", issue = "6", pages = "1363–1381",
                doi = "10.1016/j.neuron.2014.09.006",
                abstract = "Attention modulates visual processing across scales, from single neurons to receptive field remapping and neural synchrony.",
                tags = listOf("vision", "attention", "neuroscience"), projects = listOf("thesis"), reading = ReadingStatus.Unread, dateAdded = "2026-10-22",
            ),
        )
    }
}

private fun person(family: String, given: String) = Creator(sortName = family, displayName = "$family, $given")
