// library version is defined in gradle.properties
val libraryVersion: String by project

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android") version "1.6.10"
    id("maven-publish")
    id("signing")
}

repositories {
    mavenCentral()
    google()
}

android {
    compileSdk = 31

    defaultConfig {
        minSdk = 21
        targetSdk = 31
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
            proguardFiles(file("proguard-android-optimize.txt"), file("proguard-rules.pro"))
        }
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.12.0@aar")
    implementation("org.jetbrains.kotlin:kotlin-stdlib-jdk7")
    implementation("androidx.appcompat:appcompat:1.4.0")
    implementation("androidx.core:core-ktx:1.7.0")
}

afterEvaluate {
    publishing {
        publications {
            create<MavenPublication>("maven") {
                groupId = "io.github.rust-nostr"
                artifactId = "nostr"
                version = libraryVersion

                from(components["release"])
                pom {
                    name.set("nostr")
                    description.set("Nostr Kotlin language bindings.")
                    url.set("https://github.com/rust-nostr/nostr")
                    licenses {
                        license {
                            name.set("MIT")
                            url.set("https://github.com/rust-nostr/nostr/blob/master/LICENSE")
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
                        url.set("https://github.com/rust-nostr/nostr/tree/master")
                    }
                }
            }
        }
    }
}

signing {
    useGpgCmd()
    sign(publishing.publications)
}
