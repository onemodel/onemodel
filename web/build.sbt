val projectVersion = "0.2.0-SNAPSHOT"

lazy val root = (project in file(".")).
  settings(
    organization := "org.onemodel",
    name := "om-web",
    version := projectVersion,
    resolvers += Resolver.mavenLocal,
    libraryDependencies += "org.onemodel" % "core" % projectVersion,
    scalaVersion := "2.11.8"
  )
