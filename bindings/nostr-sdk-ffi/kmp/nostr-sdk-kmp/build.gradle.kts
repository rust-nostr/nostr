plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.multiplatform")
    id("com.vanniktech.maven.publish") version "0.28.0"
}

apply(plugin = "kotlinx-atomicfu")

kotlin {
    // Enable the default target hierarchy
    applyDefaultHierarchyTemplate()

    androidTarget {
        compilations.all {
            kotlinOptions {
                jvmTarget = JavaVersion.VERSION_17.majorVersion
            }
        }

        publishLibraryVariants("release")
    }

    jvm {
        compilations.all {
            kotlinOptions.jvmTarget = JavaVersion.VERSION_17.majorVersion
        }
    }

    listOf(
        iosX64(),
        iosArm64(),
        iosSimulatorArm64()
    ).forEach {
        val platform = when (it.targetName) {
            "iosSimulatorArm64" -> "ios_simulator_arm64"
            "iosArm64" -> "ios_arm64"
            "iosX64" -> "ios_x64"
            else -> error("Unsupported target $name")
        }

        it.compilations["main"].cinterops {
            create("nostrCInterop") {
                defFile(project.file("src/nativeInterop/cinterop/nostr.def"))
                includeDirs(project.file("src/nativeInterop/cinterop/headers/nostr_sdk"), project.file("src/libs/$platform"))
            }
        }
    }

    sourceSets {
        all {
            languageSettings.apply {
                optIn("kotlinx.cinterop.ExperimentalForeignApi")
            }
        }

        val commonMain by getting {
            dependencies {
                implementation(libs.okio)
                implementation(libs.kotlinx.datetime)
                implementation(libs.kotlinx.coroutines.core)
            }
        }

        val jvmMain by getting {
            dependencies {
                implementation(libs.jna)
            }
        }

        val androidMain by getting {
            dependencies {
                implementation("${libs.jna.get()}@aar")
            }
        }
    }
}

android {
    namespace = "rust.nostr.sdk"
    compileSdk = 34

    defaultConfig {
        minSdk = 21
        consumerProguardFiles("consumer-rules.pro")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

mavenPublishing {
    configure(com.vanniktech.maven.publish.AndroidMultiVariantLibrary(
        sourcesJar = true,
        publishJavadocJar = true,
    ))

    publishToMavenCentral(com.vanniktech.maven.publish.SonatypeHost.CENTRAL_PORTAL, automaticRelease = true)

    signAllPublications()

    coordinates("org.rust-nostr", "nostr-sdk-kmp", "0.38.0")

    pom {
        name.set("nostr-sdk-kmp")
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
