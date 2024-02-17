package dev.zeta.vortex


import zio.{Console, NonEmptyChunk, ZIO, ZIOAppDefault}
import zio.openai.*
import zio.openai.model.CreateChatCompletionRequest
import zio.openai.model.CreateChatCompletionRequest.Model
import zio.openai.model.ChatCompletionRequestMessage
import zio.openai.model.*
import zio.openai.model.Temperature

object ZIOOpenAI extends ZIOAppDefault  {
  def generatePrompt(animal: String): String =
    s"""Suggest three names for an animal that is a superhero.
         |
         |Animal: Cat
         |Names: Captain Sharpclaw, Agent Fluffball, The Incredible Feline
         |Animal: Dog
         |Names: Ruff the Protector, Wonder Canine, Sir Barks-a-Lot
         |Animal: ${animal.capitalize}
         |Names:""".stripMargin
  def loop = for {
    
    animal <- Console.readLine("Animal: ")
    
    resultT <- Chat.createChatCompletion(
      NonEmptyChunk[ChatCompletionRequestMessage](
        ChatCompletionRequestMessage
          .ChatCompletionRequestSystemMessage(
            ChatCompletionRequestSystemMessage(
              content = "You are a helpful assistant",
              role = ChatCompletionRequestSystemMessage.Role.System
            )
          ),
        ChatCompletionRequestMessage
          .ChatCompletionRequestUserMessage(
            ChatCompletionRequestUserMessage(
              content = ChatCompletionRequestUserMessage.Content.String(generatePrompt(animal)),
              role = ChatCompletionRequestUserMessage.Role.User,
            )
          )
      ),
      model = Model.Predefined(Model.Models.`Gpt-3.5-turbo`),
      temperature = Temperature(1.0),
    ).timed
    
    (t, result) = resultT
    
    _ <- Console.printLine(s"""${t.toMillis}ms | ${result.choices.headOption.map(_.message).mkString(", ")}""")
    
  } yield ()

  override def run =
    loop.forever.provide(Chat.default)
}