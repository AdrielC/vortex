package dev.zeta.vortex

import eu.timepit.refined.api.*
import eu.timepit.refined.collection.{NonEmpty, Size}
import eu.timepit.refined.numeric.GreaterEqual
import zio.Config.*
import zio.config.*
import zio.config.magnolia.*
import zio.config.refined.*
import zio.{Config, ConfigProvider, *}

case class ChatApiConfig(apiKey: Option[String Refined NonEmpty],
                         host: Option[String Refined NonEmpty],
                         port: Option[Int Refined GreaterEqual[1024]],
                         modelName: Option[String])
object ChatApiConfig extends App {

  val chatApiConfig = deriveConfig[ChatApiConfig]

  val source = ConfigProvider.fromEnv(pathDelim="_")

  val config = Unsafe.unsafe(implicit u =>

    Runtime.default.unsafe.run(
        source.load(chatApiConfig.mapKey {
          case "apiKey" => "OPENAI_API_KEY"
          case k => k
        }).debug)
      .getOrThrowFiberFailure()
  )
}

object ApiKeys {

  val OPENAI_API_KEY: String = ChatApiConfig.config.apiKey.map(_.value).getOrElse("")
}