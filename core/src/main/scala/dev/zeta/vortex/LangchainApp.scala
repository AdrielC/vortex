package dev.zeta.vortex

import dev.langchain4j.data.message.ChatMessage
import dev.langchain4j.data.message.ChatMessageDeserializer.messagesFromJson
import dev.langchain4j.data.message.ChatMessageSerializer.messagesToJson
import dev.langchain4j.memory.chat.{ChatMemoryProvider, MessageWindowChatMemory}
import dev.langchain4j.model.chat.ChatLanguageModel
import dev.langchain4j.model.openai.OpenAiChatModel
import dev.langchain4j.service.{AiServices, MemoryId, UserMessage}
import dev.langchain4j.store.memory.chat.{ChatMemoryStore, InMemoryChatMemoryStore}
import dev.zeta.vortex.ChatInfo.SessionId
import org.mapdb.Serializer.{INTEGER, STRING}
import org.mapdb.{DB, DBMaker}
import zio.*
import zio.Config.*
import zio.config.*

import java.util

trait Assistant {
  def chat(@MemoryId memoryId: Int,
           @UserMessage userMessage: String): String
}

object Assistant {

  val live: ZLayer[ChatLanguageModel & ChatMemoryProvider, Throwable, Assistant] =
    ZLayer {
      ZIO.environmentWithZIO[ChatLanguageModel with ChatMemoryProvider] { env =>
        ZIO.attempt {
          AiServices
            .builder(classOf[Assistant])
            .chatLanguageModel(env.get[ChatLanguageModel])
            .chatMemoryProvider(env.get[ChatMemoryProvider])
            .build
        }
      }
    }
}

// You can create your own implementation of ChatMemoryStore and store chat memory whenever you'd like
class PersistentChatMemoryStore(db: DB)
  extends ChatMemoryStore
    with AutoCloseable {

  final private val map = db.hashMap("messages", INTEGER, STRING).createOrOpen

  private val store = db.getStore
  def getMessages(memoryId: Any): util.List[ChatMessage] =
    val json: String = map.get(memoryId.asInstanceOf[Int])
    val m = messagesFromJson(json)
    println((memoryId, json))
    m

  def updateMessages(memoryId: Any, messages: util.List[ChatMessage]) =
    val json: String = messagesToJson(messages)
    map.put(memoryId.asInstanceOf[Int], json)
    db.commit

  override def deleteMessages(memoryId: Any) =
    map.remove(memoryId.asInstanceOf[Int])
    db.commit
  override def close(): Unit =
    store.compact()
    store.checkThreadSafe()
    println(s"Closing PersistentChatMemoryStore db")
    store.close()
    db.close()
}

object PersistentChatMemoryStore {
  def chatDB(fileName: String = "multi-user-chat-memory.db"): ZLayer[Any, Throwable, DB] =
    ZLayer {
      ZIO.attemptBlocking {
        DBMaker.fileDB(fileName).transactionEnable.make
      }
    }


  val inMemoryChatStore: ZLayer[Any, Nothing, InMemoryChatMemoryStore] =
    ZLayer.fromZIO(ZIO.succeed(
      new InMemoryChatMemoryStore
    ))

  val chatMemoryStore: ZLayer[DB, Throwable, PersistentChatMemoryStore] =
    ZLayer.scoped[DB] {
      ZIO.fromAutoCloseable {
        ZIO.serviceWithZIO[DB] { db =>
          ZIO.attemptBlocking {
            new PersistentChatMemoryStore(db)
          }
        }
      }
    }

  def chatMemoryProvider(maxMessages: Int = 10)
                        (implicit trace: Trace)
  : ZLayer[ChatMemoryStore, Nothing, ChatMemoryProvider] =
    ZLayer {
      ZIO.serviceWith { (store: ChatMemoryStore) =>
        ((memoryId: Any) =>
          MessageWindowChatMemory.builder
            .id(memoryId)
            .maxMessages(maxMessages)
            .chatMemoryStore(store)
            .build): ChatMemoryProvider
      }.debug("startup")
    }
}

final case class ChatConfig(apiKey: String,
                            baseUrl: Option[String],
                            modelName: Option[String])

object ChatConfig {

  val defaultProvider =
    ConfigProvider.fromEnv() orElse
      ConfigProvider.fromEnv(pathDelim="__")

  val config =
    ((string("openaiApiKey")
      .orElse(string("chatApiKey"))
      .orElse(string("key").nested("api"))
      .orElse(string("apiKey"))) zip
      string("host").optional zip
      string("modelName").optional
      ).to[ChatConfig]

  val testConfig: ZLayer[Any, Error, ChatConfig] =
    ZLayer.fromZIO(defaultProvider.load(
      config orElse config.mapKey {
        case "apiKey" => "OPENAI_API_KEY"
        case "modelName" => "MODEL_NAME"
        case o => o
      }).debug)

  val chatLanguageModel: ZLayer[ChatConfig, Throwable, ChatLanguageModel] =
    ZLayer.fromZIO {
      ZIO.serviceWithZIO { chatConfig =>
          ZIO.attempt {
            val builder = OpenAiChatModel.builder()
            val wApiKey = builder.apiKey(chatConfig.apiKey)
            val wModelName = chatConfig.modelName.fold(wApiKey)(wApiKey.modelName)
            val wBaseUrl = chatConfig.baseUrl.fold(wModelName)(wModelName.baseUrl)
            wBaseUrl.build()
          }
      }
    }
}

object LangchainApp extends ZIOApp {

  override type Environment = Assistant

  override val environmentTag: zio.EnvironmentTag[Environment] = summon

  override val bootstrap: ZLayer[Any, Throwable, Environment] =
    (PersistentChatMemoryStore.chatDB() >>>
      (PersistentChatMemoryStore.chatMemoryStore >>>
        PersistentChatMemoryStore.chatMemoryProvider())) ++
    ((ChatConfig.testConfig >>> ChatConfig.chatLanguageModel)) >>>
      Assistant.live

  def chat(message: String)
          (implicit trace: Trace): ZIO[Assistant, Any, RepliedMessage] =
    ZIO.serviceWithZIO[Assistant] { assistant =>
      for
        response <- SessionId.withSession { sessionId =>
          ZIO.attemptBlocking {
              assistant.chat(sessionId.toInt, message)
            }
            .timed.map(_.swap).debug
//            .tap(m => ZIO.debug(f"${m._1}\ntime: ${m._2.toMillis}ms\n"))
        }
        output <- RepliedMessage.make(
          Utterance.user(message),
          Utterance.system(response._1)
        )
      yield output
    }
    
    
  def consoleChat(implicit trace: Trace) =
    for
      input   <- Console.readLine("user:\n").filterOrElse(_.nonEmpty)(ZIO.interrupt)
      output  <- chat(input)
    yield (input, output)

  def run: ZIO[Assistant with ZIOAppArgs with Scope, Any, Any] =
    for
      _     <- SessionId.newSession
      _     <- chat("Hello, my name is Klaus")
      _     <- chat("Tell me what is my name")
      _     <- chat("Hi, my name is Francine")
      _     <- chat("Tell me what is my name")
      _     <- ChatInfo.SessionId.newSession
      out   <- chat("Tell me what is my name")
      _     <- consoleChat.repeat(Schedule.forever)
    yield out
}