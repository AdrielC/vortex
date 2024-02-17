package dev.zeta.vortex

import dev.langchain4j.chain.ConversationalRetrievalChain
import dev.langchain4j.data.document.loader.FileSystemDocumentLoader.loadDocument
import dev.langchain4j.data.document.parser.TextDocumentParser
import dev.langchain4j.data.document.splitter.DocumentSplitters
import dev.langchain4j.data.document.{Document, DocumentSplitter}
import dev.langchain4j.data.embedding.Embedding
import dev.langchain4j.data.message.AiMessage
import dev.langchain4j.data.segment.TextSegment
import dev.langchain4j.model.chat.ChatLanguageModel
import dev.langchain4j.model.embedding.{AllMiniLmL6V2EmbeddingModel, EmbeddingModel}
import dev.langchain4j.model.input.{Prompt, PromptTemplate}
import dev.langchain4j.model.openai.OpenAiChatModelName.{GPT_3_5_TURBO}
import dev.langchain4j.rag.content.retriever.EmbeddingStoreContentRetriever
import dev.langchain4j.model.openai.{OpenAiChatModel, OpenAiTokenizer}
import dev.langchain4j.retriever.EmbeddingStoreRetriever
import dev.langchain4j.store.embedding.inmemory.InMemoryEmbeddingStore
import dev.langchain4j.store.embedding.{EmbeddingMatch, EmbeddingStore, EmbeddingStoreIngestor}

import java.net.{URI, URL}
import java.nio.file.{Path, Paths}
import java.time.{Duration, Instant}
import scala.jdk.CollectionConverters.*
import scala.util.Try

object ChatWithDocumentsExamples extends App {
  def toPath(fileName: String): Try[Path] = Try {
    val fileUrl = getClass.getResource(fileName)
    val file = Paths.get(fileName).toUri
    println((fileName, fileUrl, getClass, file))
    println(file)
    Paths.get(file)
  }

  object IfYouNeedSimplicity {
    val embeddingModel: EmbeddingModel = new AllMiniLmL6V2EmbeddingModel

    val embeddingStore: EmbeddingStore[TextSegment] = new InMemoryEmbeddingStore[TextSegment]()

    val ingestor: EmbeddingStoreIngestor = EmbeddingStoreIngestor.builder()
      .documentSplitter(DocumentSplitters.recursive(300, 0))
      .embeddingModel(embeddingModel)
      .embeddingStore(embeddingStore)
      .build()

    val document: Document = loadDocument(toPath(
      "core/src/main/resources/example-files/story-about-happy-carrot.txt"
    ).get, new TextDocumentParser)
    ingestor.ingest(document)

    val chain: ConversationalRetrievalChain = ConversationalRetrievalChain.builder()
      .chatLanguageModel(OpenAiChatModel.withApiKey(ApiKeys.OPENAI_API_KEY))
      .contentRetriever(EmbeddingStoreContentRetriever.builder
        .embeddingStore(embeddingStore)
        .embeddingModel(embeddingModel)
        .build())
      .build()

    val answer: String = chain.execute("Who is Charlie?")
    println(answer) // Charlie is a cheerful carrot living in VeggieVille...
  }


  val document: Document = loadDocument(toPath("core/src/main/resources/example-files/story-about-happy-carrot.txt").get, new TextDocumentParser)

  val splitter: DocumentSplitter = DocumentSplitters.recursive(
    100,
    0,
    new OpenAiTokenizer(GPT_3_5_TURBO.toString())
  )
  val segments: List[TextSegment] = splitter.split(document).asScala.toList

  val embeddingModel: EmbeddingModel = new AllMiniLmL6V2EmbeddingModel
  val embeddings: List[Embedding] = embeddingModel.embedAll(segments.asJava).content().asScala.toList

  val embeddingStore: EmbeddingStore[TextSegment] = new InMemoryEmbeddingStore[TextSegment]()
  embeddingStore.addAll(embeddings.asJava, segments.asJava)

  val question: String = "Who is Charlie?"

  val questionEmbedding: Embedding = embeddingModel.embed(question).content()

  val maxResults = 3
  val minScore = 0.7
  val relevantEmbeddings: List[EmbeddingMatch[TextSegment]] =
    embeddingStore.findRelevant(questionEmbedding, maxResults, minScore).asScala.toList

  val promptTemplate: PromptTemplate = PromptTemplate.from(
    """Answer the following question to the best of your ability:
      |
      |Question:
      |{{question}}
      |
      |Base your answer on the following information:
      |{{information}}""".stripMargin)


  val chatModel: ChatLanguageModel = OpenAiChatModel.builder()
    .apiKey(ApiKeys.OPENAI_API_KEY)
    .timeout(Duration.ofSeconds(60))
    .build()
  
  val aiMessage: AiMessage = LazyList.fill(100) {

    val questionEmbedding: Embedding = embeddingModel.embed(question).content()

    val maxResults = 3
    val minScore = 0.7
    val relevantEmbeddings: List[EmbeddingMatch[TextSegment]] =
      embeddingStore.findRelevant(questionEmbedding, maxResults, minScore).asScala.toList

    val information: String = relevantEmbeddings.map(_.embedded().text()).mkString("\n\n")
    val variables: Map[String, Any] = Map("question" -> question, "information" -> information)
    val prompt: Prompt = promptTemplate.apply(variables.asJava)

    val t1 = Instant.now()
    val res = chatModel.generate(prompt.toUserMessage()).content()
    val t2 = Instant.now()
    val dur = t2.toEpochMilli - t1.toEpochMilli
    println(s"${dur} ms")
    res
  }.toList.last

  val answer: String = aiMessage.text()
  println(answer) // Charlie is a cheerful carrot living in VeggieVille...
}

