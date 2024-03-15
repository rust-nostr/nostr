// library version is defined in gradle.properties
val libraryVersion: String by project

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android") version "1.9.22"
    id("maven-publish")
    id("signing")
}

repositories {
    mavenCentral()
    google()
}

android {
    namespace = "rust.nostr.protocol"
    
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

    publishing {
        singleVariant("release") {
            withSourcesJar()
            withJavadocJar()
        }
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.12.0@aar")
    implementation("androidx.appcompat:appcompat:1.6.1")
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
