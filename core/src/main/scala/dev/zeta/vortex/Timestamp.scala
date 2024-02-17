package dev.zeta.vortex


import zio.schema.Schema

case class Timestamp(value: Long) {
  def <(other: Timestamp): Boolean     = value < other.value
  def <=(other: Timestamp): Boolean    = value <= other.value
  def >(other: Timestamp): Boolean     = value > other.value
  def >=(other: Timestamp): Boolean    = value >= other.value
  def max(other: Timestamp): Timestamp = Timestamp(Math.max(value, other.value))
  def min(other: Timestamp): Timestamp = Timestamp(Math.min(value, other.value))

  def next: Timestamp = Timestamp(value + 1)
}

object Timestamp {
  implicit val schema: Schema[Timestamp] = Schema[Long].transform(
    Timestamp(_),
    _.value
  )

  implicit val ordering: Ordering[Timestamp] = (x: Timestamp, y: Timestamp) => x.value.compareTo(y.value)
}