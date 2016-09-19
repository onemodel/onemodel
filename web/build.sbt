val projectVersion = "0.2.0-SNAPSHOT"

lazy val root = (project in file(".")).
  settings(
    organization := "org.onemodel",
    name := "om-web",
    version := projectVersion,
    scalaVersion := "2.11.8",
    resolvers += Resolver.mavenLocal,
    libraryDependencies += "org.onemodel" % "core" % projectVersion
  ).
  enablePlugins(PlayScala)

// cached resolution for performance on multiple subprojects. Docs said
// is experimental.  If issues, remove, and/or ck http://www.scala-sbt.org/1.0/docs/Cached-Resolution.html .
updateOptions := updateOptions.value.withCachedResolution(true)
