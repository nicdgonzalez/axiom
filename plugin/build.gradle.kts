plugins {
    application
}

repositories {
    mavenCentral()
    maven("https://repo.papermc.io/repository/maven-public/")
}

dependencies {
    compileOnly("io.papermc.paper:paper-api:1.21.1-R0.1-SNAPSHOT")
}

java {
    toolchain.languageVersion.set(JavaLanguageVersion.of(21))
}

application {
    mainClass = "io.github.nicdgonzalez.axiom.Axiom"
}

tasks.processResources {
    filesMatching("plugin.yml") {
        expand(
            "name" to project.name,
            "version" to version,
        )
    }
}
