package dev.zeta

import zio._
import zio.prelude.Assertion._
import zio.prelude.Newtype
import zio.schema.Schema
import dev.zeta.vortex.FlowId.unwrap

import java.nio.charset.StandardCharsets
import java.util.UUID

package object vortex {

  type RemoteVariableName = RemoteVariableName.Type

  object RemoteVariableName extends Newtype[String] {
    implicit val schema: Schema[RemoteVariableName] = Schema[String].transform(wrap(_), unwrap)

    override def assertion = {
      !contains("__")
    }

    def unsafeMake(name: String): RemoteVariableName = wrap(name)
  }

  type BindingName = BindingName.Type

  object BindingName extends Newtype[UUID] {
    implicit val schema: Schema[BindingName] = Schema[UUID].transform(wrap(_), unwrap)
  }

  type FlowId = FlowId.Type

  object FlowId extends Newtype[String] {
    implicit val schema: Schema[FlowId] = Schema[String].transform(wrap(_), unwrap)

    override def assertion = {
      !contains("__")
    }

    def unsafeMake(name: String): FlowId = wrap(name)

    def newRandom: ZIO[Any, Nothing, FlowId] =
      zio.Random.nextUUID.map(rid => wrap(rid.toString))
  }

  implicit class FlowIdSyntax(val flowId: FlowId) extends AnyVal {
    def /(postfix: FlowId): FlowId = FlowId.unsafeMake(unwrap(flowId) + "_" + unwrap(postfix))

    def toRaw: Chunk[Byte] = Chunk.fromArray(unwrap(flowId).getBytes(StandardCharsets.UTF_8))
  }

  type TransactionId = TransactionId.Type

  object TransactionId extends Newtype[String] {
    implicit val schema: Schema[TransactionId] = Schema[String].transform(wrap(_), unwrap)

    override def assertion = {
      !contains("__")
    }

    def fromCounter(counter: Int): TransactionId =
      wrap(counter.toString)

    def unsafeMake(name: String): TransactionId =
      wrap(name)
  }

  type RecursionId = RecursionId.Type

  object RecursionId extends Newtype[UUID] {
    implicit val schema: Schema[RecursionId] = Schema[UUID].transform(wrap(_), unwrap)
  }

  implicit class RecursionIdSyntax(val recursionId: RecursionId) extends AnyVal {
    def toRemoteVariableName: RemoteVariableName =
      RemoteVariableName.unsafeMake(s"_!rec!_${RecursionId.unwrap(recursionId)}")
  }

  type PromiseId = PromiseId.Type

  object PromiseId extends Newtype[String] {
    implicit val schema: Schema[PromiseId] = Schema[String].transform(wrap(_), unwrap)
  }

  type ConfigKey = ConfigKey.Type

  object ConfigKey extends Newtype[String] {
    implicit val schema: Schema[ConfigKey] = Schema[String].transform(wrap(_), unwrap)
  }

}
