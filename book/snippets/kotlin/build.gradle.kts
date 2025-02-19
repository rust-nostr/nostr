plugins {
    kotlin("jvm") version "2.1.0"
}

group = "rust.nostr.snippets"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
    implementation("org.rust-nostr:nostr-sdk-jvm:0.39.0")
}

tasks.test {
    useJUnitPlatform()
}
kotlin {
    jvmToolchain(17)
}
