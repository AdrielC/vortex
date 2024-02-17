package dev.zeta.vortex


import zio.prelude.Hash
import zio.{Chunk, FiberRef, Scope, Trace, Unsafe, ZEnvironment, ZIO}

import java.util.UUID
import scala.util.hashing.MurmurHash3

object ChatInfo {
  
  opaque type SessionId = Int
  object SessionId:

    extension (s: SessionId)
      def toInt: Int = s.self
    
    private final val sessionId = Unsafe.unsafe(implicit u => FiberRef.unsafe.make[SessionId](0))
    private final val sessionEnv = Unsafe.unsafe(implicit u => 
      FiberRef.unsafe.makeEnvironment(ZEnvironment(Chunk.empty[Utterance[_]])))
    def apply[Value](v: Value)(using h: Hash[Value]): SessionId = h.hash(v)
    def newSession(implicit trace: Trace): ZIO[Scope, Nothing, SessionId] =
      for
        s <- ZIO.randomWith(_.nextUUID)
        id = SessionId(s.hashCode())
        _ <- sessionId.locallyScoped(id)
      yield id

    def currentSession(implicit trace: Trace): ZIO[Any, Nothing, SessionId] =
      for
        id <- sessionId.get
      yield id
    def withSession[R, E, A](_zio: SessionId => ZIO[R, E, A])
                            (implicit trace: Trace) =
      for
        sessionId <- sessionId.get
        out <- ZIO.logAnnotate("sessionId", sessionId.toString) {
          _zio(sessionId)
        }
      yield out
    def withCurrent[R, E, A](_zio: SessionId => ZIO[R, E, A]) =
      for 
        sessionId <- sessionId.get 
        out <- ZIO.logAnnotate("sessionId", sessionId.toString) {
          _zio(sessionId)
        }
      yield out

  val maxHistory = Unsafe.unsafe(implicit u => FiberRef.unsafe.make(20))
  
  val chatHistory = Unsafe.unsafe(implicit u => FiberRef.unsafe.make[(Chunk[RepliedMessage], Int)](
    (Chunk.empty[RepliedMessage], maxHistory.initial),
    fork = a => (a._1.takeRight(a._2), a._2),
    join = (a, b) => (a._1 ++ b._1).sortBy(_.timestamp).takeRight(a._2 min b._2) -> (a._2 min b._2)
  ))
}