package dev.zeta.vortex

import dev.zeta.vortex.Main.Environment
import zio.*
import zio.Console.printLine
import zio.stream.*

object Main extends ZIOAppDefault:
  override def run: ZIO[Environment & ZIOAppArgs & Scope, Any, Any] =

    // Sealed means the trait’s extensions must be in the
    // same file

    // Our own custom data structure
    sealed trait MyList[+A] {
      def map[B](f: A => B): MyList[B]
    }


    // Another "interface"
    type Price = Double
    trait MarketDataTopic {
      def subscribe(ticker: String): ZStream[Any, Throwable, Price]
    }

    // Returning a value that, if ran, will return an int
    // 
    val infallible: ZIO[Any, Throwable, Int] = {
      ZIO.attempt {
        throw new Throwable("Boom!")
        println("hello")
        1
      }
    }
    
      
    
    // ZIO[R (requirements), E (error), A (result)]
    def connect(user: String, password: String): ZIO[Any, Throwable, DBConn] =
    // similar to try/catch
      ZIO.attempt {
        ???
      }
    





    trait Logger {

      def log(message: String): ZIO[Any, Nothing, Unit]
    }



    type DBConn = Any

    // Different implementations for how it will work in production
    // Or one for unit testing where the dependencies are not available
    case class LiveMarketData(username: String, password: String, conn: DBConn)
      extends MarketDataTopic {

      override def subscribe(ticker: String): ZStream[Any, Throwable, Price] = ???
    }


    def process(topic: MarketDataTopic, ticker: String) =
      topic
        .subscribe(ticker)
        .runSum


    object TestTopic extends MarketDataTopic {
      override def subscribe(ticker: String): ZStream[Any, Throwable, Price] =
        ZStream.repeat(1.0).take(20)
    }


    process(TestTopic, "AAPL")
    process(LiveMarketData("Miki", "pass", 1), "AAPL")


    // Case 1: Non-empty
    case class Cons[+A](head: A, tail: MyList[A]) extends MyList[A] {

      override def map[B](f: A => B): MyList[B] =
        Cons(f(head), tail.map(f))
    }

    // Case 2: Empty (Nil can be named anything)
    case object Nil extends MyList[Nothing] {
      override def map[B](f: Nothing => B): MyList[B] =
        Nil
    }

    object MyList {
      // The apply() method of an object is special in Scala.
      // Its name doesn't need to be called in order to use it
      // (simply use the object name and parentheses).
      def apply[A](a: A*): MyList[A] = ???
    }

    val myList: MyList[Int] = MyList(1, 2, 3)

    def sum(myList: MyList[Int]): Int =
      myList match
        case Cons(head, tail) => head + sum(tail)
        case Nil => 0


    printLine("Welcome to your first ZIO app!")


