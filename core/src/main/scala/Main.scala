import zio.*
import zio.Console.printLine
import zio.stream.*

object Main extends ZIOAppDefault:
  override def run: ZIO[Environment & ZIOAppArgs & Scope, Any, Any] =

    val stream = ZStream(1, 2, 3, 4, 5).forever.take(100)

    trait Model {
      def compile: Double => Double
    }

    case class Wavelet() extends Model {

      override def compile: Double => Double =
        x => x + 1
    }

    val model: Model = Wavelet()

    val fn = model.compile

    val program = stream
      .map(x => fn(x))
      .tap(x => printLine(x))
      .scan(1.0)((state, value) => state + value)
      .runCollect

    printLine("Welcome to your first ZIO app!") *>
      program



