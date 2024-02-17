package dev.zeta.vortex


import dev.zeta.vortex.Utterance.Role

import java.time.{Instant, OffsetDateTime}

case class Utterance[+R <: Role](role: R, message: String) {
  override def toString: String =
    s"""$role:\n$message"""
}
object Utterance:
  inline transparent def user(message: String): Utterance[Role] =
    apply("user", message)
  inline transparent def system(message: String): Utterance[Role] =
    apply("system", message)
  inline transparent def tool(message: String): Utterance[Role] =
    apply("tool", message)
  inline transparent def assistant(message: String): Utterance[Role] =
    apply("assistant", message)

  type Role = User | NonUser
  type User = "user"
  type NonUser = "system" | "tool" | "assistant"
