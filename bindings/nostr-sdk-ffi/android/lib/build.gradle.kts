plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android") version "1.9.22"
    id("com.vanniktech.maven.publish") version "0.28.0"
    id("signing")
}

repositories {
    mavenCentral()
    google()
}

android {
    namespace = "rust.nostr.sdk"

    compileSdk = 34

    defaultConfig {
        minSdk = 21

        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(file("proguard-android-optimize.txt"), file("proguard-rules.pro"))
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    kotlinOptions {
        jvmTarget = "1.8"
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.15.0@aar")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
    implementation("androidx.appcompat:appcompat:1.6.1")
}

mavenPublishing {
    configure(com.vanniktech.maven.publish.AndroidMultiVariantLibrary(
        sourcesJar = true,
        publishJavadocJar = true,
    ))

    publishToMavenCentral(com.vanniktech.maven.publish.SonatypeHost.CENTRAL_PORTAL, automaticRelease = true)

    signAllPublications()

    coordinates("org.rust-nostr", "nostr-sdk", "0.40.0")

    pom {
      name.set("nostr-sdk")
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
