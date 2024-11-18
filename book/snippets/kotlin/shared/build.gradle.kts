plugins {
    alias(libs.plugins.kotlinAndroid)
    alias(libs.plugins.androidLibrary)
}

android {
    namespace = "rust.nostr.snippets"

    compileSdk = 34

    defaultConfig {
        minSdk = 34
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
    implementation(libs.nostr)
    implementation("androidx.core:core-ktx:1.12.0")
}