package dev.zeta.vortex

import dev.zeta.vortex.ChatInfo.SessionId
import dev.zeta.vortex.Utterance.{NonUser, User}
import zio.{NonEmptyChunk, Trace, ZIO}

import java.time.Instant

case class RepliedMessage(sessionId: Int,
                          inputMessage: Utterance[User],
                          reply: NonEmptyChunk[Utterance[NonUser]],
                          timestamp: Instant) {
  override def toString: String =
    s"""
       |$inputMessage
       |${reply.mkString("\n")}
       |""".stripMargin
}

object RepliedMessage {
  def make(inputMessage: Utterance[User],
           reply: Utterance[NonUser])
          (implicit trace: Trace) =
    ZIO.clockWith(_.instant)
      .zipWithPar(SessionId.currentSession)(
        (t, s) => RepliedMessage(
          s.toInt,
          inputMessage=inputMessage,
          reply=NonEmptyChunk.single(reply),
          timestamp = t
        )
      )
}