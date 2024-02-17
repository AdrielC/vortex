import BuildHelper._

organization     := "dev.zeta"
scalaVersion     := Version.Scala3

inThisBuild(
  List(
    organization     := "dev.zeta",
    scalaVersion     := Version.Scala3,
    version          := "0.1.0-SNAPSHOT",
    homepage     := Some(url("https://zio.dev/zio-flow/")),
    licenses     := List("Apache-2.0" -> url("http://www.apache.org/licenses/LICENSE-2.0")),
    developers := List(

    ),
    resolvers +=
      "Sonatype OSS Snapshots 01" at "https://s01.oss.sonatype.org/content/repositories/snapshots",
    pgpPassphrase := sys.env.get("PGP_PASSWORD").map(_.toArray),
    pgpPublicRing := file("/tmp/public.asc"),
    pgpSecretRing := file("/tmp/secret.asc"),
    resolvers +=
      "Sonatype OSS Snapshots" at "https://oss.sonatype.org/content/repositories/snapshots"
  )
)

addCommandAlias("fmt", "all scalafmtSbt scalafmtAll")
addCommandAlias("fmtCheck", "all scalafmtSbtCheck scalafmtCheckAll")


lazy val commonTestDependencies =
  Seq(
    "dev.zio" %% "zio-test"     % Version.zio,
    "dev.zio" %% "zio-test-sbt" % Version.zio
  )

lazy val zioTest = new TestFramework("zio.test.sbt.ZTestFramework")

lazy val root = project
  .in(file("."))
  .settings(
    publish / skip := true
  )
  .aggregate(
//    vortexJVM,
//    vortexJS,
    vortex
  )
//
//lazy val vortex =
//  crossProject(JSPlatform, JVMPlatform)
//    .in(file("."))
//    .settings(stdSettings("vortex"))
//    .settings(crossProjectSettings)
//    .settings(buildInfoSettings("dev.zeta.vortex"))
//    .settings(
//      libraryDependencies ++= Seq(
//        "dev.zio" %% "zio" % Version.zio,
//        "dev.zio" %% "zio-streams" % Version.zio,
//        "dev.zio" %% "zio-config"          % Version.zioConfig,
//        "dev.zio" %% "zio-config-magnolia" % Version.zioConfig,
//        "dev.zio" %% "zio-config-typesafe" % Version.zioConfig,
//        "dev.zio" %% "zio-config-refined"  % Version.zioConfig,
//        "dev.zio" %% "zio-test" % Version.zio % Test,
//        "dev.zio"     %% "zio-metrics-connectors"   % Version.zioMetricsConnectors,
//
//        "dev.zio" %% "zio-http"              % Version.zioHttp,
//        "dev.zio" %% "zio-prelude"           % Version.zioPrelude,
//        "dev.zio" %% "zio-schema"            % Version.zioSchema,
//        "dev.zio" %% "zio-schema-derivation" % Version.zioSchema,
//        "dev.zio" %% "zio-schema-optics"     % Version.zioSchema,
//        "dev.zio" %% "zio-schema-json"       % Version.zioSchema,
//        "dev.zio" %% "zio-schema-protobuf"   % Version.zioSchema,
//
//        "dev.zio"     %% "zio-logging"              % Version.zioLogging,
//        "dev.zio"     %% "zio-logging-slf4j-bridge" % Version.zioLogging,
//
//        "com.typesafe" % "config"                   % Version.config,
//
//        "dev.langchain4j" % "langchain4j-embeddings" % Version.langchain4j,
//        "dev.langchain4j" % "langchain4j-embeddings-all-minilm-l6-v2" % Version.langchain4j,
//        "dev.langchain4j" % "langchain4j-document-parser-apache-pdfbox" % Version.langchain4j,
//        "dev.langchain4j" % "langchain4j-document-parser-apache-poi" % Version.langchain4j,
//        "dev.langchain4j" % "langchain4j-open-ai" % Version.langchain4j,
//        "dev.langchain4j" % "langchain4j" % Version.langchain4j,
//        "dev.langchain4j" % "langchain4j-core" % Version.langchain4j,
//
//        "org.mapdb" % "mapdb" % "3.0.9",
//      ) ++
//        commonTestDependencies.map(_ % Test)
//    )
//    .settings(fork := false)
//    .settings(testFrameworks += zioTest)
//
//lazy val vortexJS = vortex.js
//  .settings(scalaJSUseMainModuleInitializer := true)
//
//lazy val vortexJVM = vortex.jvm

lazy val vortex = (project in file("core"))
//  .settings(stdSettings("vortex"))
//  .settings(crossProjectSettings)
//  .settings(buildInfoSettings("vortex"))
  .settings(
    name := "vortex",
    libraryDependencies ++= Seq(
      "dev.zio" %% "zio" % Version.zio,
      "dev.zio" %% "zio-streams" % Version.zio,
      "dev.zio" %% "zio-config"          % Version.zioConfig,
      "dev.zio" %% "zio-config-magnolia" % Version.zioConfig,
      "dev.zio" %% "zio-config-typesafe" % Version.zioConfig,
      "dev.zio" %% "zio-config-refined"  % Version.zioConfig,
      "dev.zio"     %% "zio-metrics-connectors"   % Version.zioMetricsConnectors,
      "dev.zio" %% "zio-cache" % "0.2.3",
      "dev.zio" % "zio-openai_3" % "0.4.1",

      "dev.zio" %% "zio-http"              % Version.zioHttp,
      "dev.zio" %% "zio-prelude"           % Version.zioPrelude,
      "dev.zio" %% "zio-schema"            % Version.zioSchema,
      "dev.zio" %% "zio-schema-derivation" % Version.zioSchema,
      "dev.zio" %% "zio-schema-optics"     % Version.zioSchema,
      "dev.zio" %% "zio-schema-json"       % Version.zioSchema,
      "dev.zio" %% "zio-schema-protobuf"   % Version.zioSchema,

      "dev.zio"     %% "zio-logging"              % Version.zioLogging,
      "dev.zio"     %% "zio-logging-slf4j-bridge" % Version.zioLogging,

      "com.typesafe" % "config"                   % Version.config,

      "dev.langchain4j" % "langchain4j-embeddings" % Version.langchain4j,
      "dev.langchain4j" % "langchain4j-embeddings-all-minilm-l6-v2" % Version.langchain4j,
      "dev.langchain4j" % "langchain4j-document-parser-apache-pdfbox" % Version.langchain4j,
      "dev.langchain4j" % "langchain4j-document-parser-apache-poi" % Version.langchain4j,
      "dev.langchain4j" % "langchain4j-open-ai" % Version.langchain4j,
      "dev.langchain4j" % "langchain4j" % Version.langchain4j,
      "dev.langchain4j" % "langchain4j-core" % Version.langchain4j,

      "org.mapdb" % "mapdb" % "3.0.9",
    ) ++ commonTestDependencies.map(_ % Test),
    testFrameworks += new TestFramework("zio.test.sbt.ZTestFramework")
  )
