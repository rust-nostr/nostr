import org.gradle.api.tasks.testing.logging.TestExceptionFormat.*
import org.gradle.api.tasks.testing.logging.TestLogEvent.*
import com.vanniktech.maven.publish.KotlinJvm
import com.vanniktech.maven.publish.JavadocJar

plugins {
    id("org.jetbrains.kotlin.jvm")
    id("org.gradle.java-library")
    id("org.jetbrains.dokka")
    id("com.vanniktech.maven.publish") version "0.30.0"
    id("signing")
}

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
    withSourcesJar()
    withJavadocJar()
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
    kotlinOptions {
        jvmTarget = "11"
    }
}

testing {
    suites {
        val test by getting(JvmTestSuite::class) {
            useKotlinTest("1.9.23")
        }
    }
}

tasks.withType<Test> {
    testLogging {
        events(PASSED, SKIPPED, FAILED, STANDARD_OUT, STANDARD_ERROR)
        exceptionFormat = FULL
        showExceptions = true
        showStackTraces = true
        showCauses = true
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.15.0")
    implementation(platform("org.jetbrains.kotlin:kotlin-bom"))
    implementation("org.jetbrains.kotlin:kotlin-stdlib-jdk7")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
    api("org.slf4j:slf4j-api:1.7.30")

    testImplementation("ch.qos.logback:logback-classic:1.2.3")
    testImplementation("ch.qos.logback:logback-core:1.2.3")
}

mavenPublishing {
    configure(KotlinJvm(
        javadocJar = JavadocJar.None(),
        sourcesJar = true,
      ))

    publishToMavenCentral(com.vanniktech.maven.publish.SonatypeHost.CENTRAL_PORTAL, automaticRelease = true)

    signAllPublications()

    coordinates("org.rust-nostr", "nostr-sdk-jvm", "0.40.0")

    pom {
      name.set("nostr-sdk-jvm")
      description.set("Nostr protocol implementation, Relay, RelayPool, high-level client library, NWC client and more.")
      url.set("https://rust-nostr.org")
      licenses {
          license {
              name.set("MIT")
              url.set("https://rust-nostr.org/license")
          }
      }
      developers {
          developer {
              id.set("yukibtc")
              name.set("Yuki Kishimoto")
              email.set("yukikishimoto@protonmail.com")
          }
      }
      scm {
          connection.set("scm:git:github.com/rust-nostr/nostr.git")
          developerConnection.set("scm:git:ssh://github.com/rust-nostr/nostr.git")
          url.set("https://github.com/rust-nostr/nostr")
      }
    }
}

signing {
    useGpgCmd()
}
