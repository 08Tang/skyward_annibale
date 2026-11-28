@file:Suppress("UnstableApiUsage")

import com.google.protobuf.gradle.id
import java.io.File
import java.net.HttpURLConnection
import java.net.URI

plugins {
    alias(libs.plugins.agp.app)
    alias(libs.plugins.compose.compiler)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.protobuf)
    id("kotlin-parcelize")
}

val androidCompileSdkVersion = rootProject.extra["androidCompileSdkVersion"] as Int
val androidCompileSdkVersionMinor = rootProject.extra["androidCompileSdkVersionMinor"] as Int
val androidBuildToolsVersion = rootProject.extra["androidBuildToolsVersion"] as String
val androidMinSdkVersion = rootProject.extra["androidMinSdkVersion"] as Int
val androidTargetSdkVersion = rootProject.extra["androidTargetSdkVersion"] as Int
val androidSourceCompatibility = rootProject.extra["androidSourceCompatibility"] as JavaVersion
val androidTargetCompatibility = rootProject.extra["androidTargetCompatibility"] as JavaVersion
val managerVersionCode = rootProject.extra["managerVersionCode"] as Int
val managerVersionName = rootProject.extra["managerVersionName"] as String

val isPrBuild = project.findProperty("IS_PR_BUILD")?.toString()?.toBoolean() ?: false
val defaultManagerPackageName = if (isPrBuild) "com.tkexig.skyward.pr" else "com.tkexig.skyward"
val defaultManagerName = if (isPrBuild) "Skyward Manager PR" else "Skyward Manager"
val managerPackageName = project.findProperty("KSU_PACKAGE_NAME")?.toString() ?: defaultManagerPackageName
val managerName = project.findProperty("KSU_NAME")?.toString() ?: defaultManagerName

// 签名：证书不进仓库，CI 从 secret 还原成文件后把路径传进来。
//
// v1 + v2 + v3 全开：v1 给 Android 6 及更老的系统认，v2/v3 给新系统认，
// v3 还带密钥轮换信息。三个都签上，同一份 APK 才能在各代系统上都装得上、
// 且能覆盖安装旧版本。
val keystorePath: String? = (project.findProperty("KEYSTORE_FILE") as String?)
    ?: System.getenv("KEYSTORE_FILE")
val keystorePassword: String? = (project.findProperty("KEYSTORE_PASSWORD") as String?)
    ?: System.getenv("KEYSTORE_PASSWORD")
val keyAliasValue: String? = (project.findProperty("KEY_ALIAS") as String?)
    ?: System.getenv("KEY_ALIAS")
val keyPasswordValue: String? = (project.findProperty("KEY_PASSWORD") as String?)
    ?: System.getenv("KEY_PASSWORD")

val hasKeystore = !keystorePath.isNullOrBlank() && File(keystorePath).isFile

android {
    signingConfigs {
        if (hasKeystore) {
            create("release") {
                storeFile = File(keystorePath!!)
                storePassword = keystorePassword
                keyAlias = keyAliasValue
                keyPassword = keyPasswordValue
                enableV1Signing = true
                enableV2Signing = true
                enableV3Signing = true
            }
        }
    }
}

protobuf {
    protoc {
        artifact = libs.protobuf.protoc.get().toString()
    }
    generateProtoTasks {
        ofNonTest().forEach { task ->
            task.builtins {
                id("java") {
                    option("lite")
                }
                id("kotlin") {
                    option("lite")
                }
            }
        }
    }
}


android {
    namespace = "com.tkexig.skyward"

    buildTypes {
        debug {
        }
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            vcsInfo.include = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            // 刻意不让 AGP 签名。minSdk 24 时它只出 v2/v3，而 v1 也要有；
            // 让 CI 用 apksigner 对着未签名的包一次签全三种，比事后重签
            // 干净——已带 v2 块的包再签不会补出 v1。
        }
    }

    buildFeatures {
        aidl = true
        buildConfig = true
        resValues = true
        compose = true
    }

    packaging {
        dex {
            useLegacyPackaging = true
        }
        jniLibs {
            useLegacyPackaging = true
            excludes += "lib/*/libandroidx.graphics.path.so"
        }
    }

    dependenciesInfo {
        includeInApk = false
        includeInBundle = false
    }

    androidResources {
        generateLocaleConfig = true
    }
    compileSdk {
        version =
            release(androidCompileSdkVersion) {
                minorApiLevel = androidCompileSdkVersionMinor
            }
    }
    buildToolsVersion = androidBuildToolsVersion

    defaultConfig {
        minSdk = androidMinSdkVersion
        targetSdk = androidTargetSdkVersion
        versionCode = managerVersionCode
        versionName = managerVersionName
        applicationId = managerPackageName

        buildConfigField("boolean", "IS_PR_BUILD", isPrBuild.toString())
        // CI 里一定有 git；本地解压出来的源码包可能没有，别让版本号构建挂掉。
        val commit = runCatching {
            providers.exec { commandLine("git", "rev-parse", "HEAD") }
                .standardOutput.asText.get().trim()
        }.getOrDefault("")
        buildConfigField("String", "MANAGER_COMMIT", "\"$commit\"")
        resValue("string", "app_name", managerName)

    }

    lint {
        abortOnError = true
        checkReleaseBuilds = false
    }

    compileOptions {
        isCoreLibraryDesugaringEnabled = true
        sourceCompatibility = androidSourceCompatibility
        targetCompatibility = androidTargetCompatibility
    }
}

androidComponents {
    onVariants(selector().withBuildType("release")) {
        it.packaging.resources.excludes.addAll(listOf(
            "META-INF/**",
            "kotlin/**",
            "**.bin",
            // Strip protobuf source files bundled by the protobuf runtime jar
            "google/**",
            "src/**",
        ))
    }
}

base {
    archivesName.set(
        "${managerName.replace(" ", "_")}_${managerVersionName}_${managerVersionCode}"
    )
}

dependencies {
    // java.time and java.nio.file are used throughout the manager but only exist
    // from Android 8.0; desugaring keeps them available on Android 7.
    coreLibraryDesugaring(libs.desugar.jdk.libs.nio)

    // Stable bridge shared with the optional assistant AAR.
    implementation(libs.androidx.exifinterface)

    // The assistant itself is optional: a drop-in app/libs/ai.aar wins,
    // otherwise the :ai module is used when it is part of the build. Without
    // either, the host compiles and AiFeatureBridge hides every AI entry.
    val aiAar = file("libs/ai.aar")
    val aiProject = findProject(":ai")
    if (aiAar.isFile) {
        implementation(files(aiAar))
    } else if (aiProject != null) {
        implementation(project(":ai"))
    }

    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.core.splashscreen)

    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.material.icons.core)
    implementation(files("libs/material-icons-custom.jar"))
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.documentfile)

    debugImplementation(libs.androidx.compose.ui.test.manifest)
    debugImplementation(libs.androidx.compose.ui.tooling)

    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.navigationevent.compose)

    implementation(libs.com.github.topjohnwu.libsu.core)
    implementation(libs.com.github.topjohnwu.libsu.service)
    implementation(libs.com.github.topjohnwu.libsu.io)

    implementation(libs.dev.rikka.rikkax.parcelablelist)

    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.kotlinx.serialization.json)

    implementation(libs.commonmark)
    implementation(libs.commonmark.ext.gfm.tables)
    implementation(libs.commonmark.ext.gfm.strikethrough)
    implementation(libs.commonmark.ext.autolink)
    implementation(libs.commonmark.ext.task.list.items)

    implementation(libs.androidx.webkit)


    implementation(libs.hiddenapibypass)

    implementation(libs.miuix.ui)
    implementation(libs.miuix.icons)
    implementation(libs.miuix.nav)
    implementation(libs.miuix.preference)
    implementation(libs.miuix.blur)

    implementation(libs.reorderable)

    implementation(platform(libs.okhttp.bom))
    implementation(libs.okhttp)

    implementation(libs.material.kolor)

    implementation(libs.appiconloader)

    implementation(libs.commons.compress)
    implementation(libs.xz)
    implementation(libs.protobuf.kotlin.lite)
    implementation(libs.coil.compose)

    testImplementation(libs.junit)
}

kotlin {
    compilerOptions {
        freeCompilerArgs.addAll(
            "-opt-in=androidx.compose.material3.ExperimentalMaterial3Api",
            "-opt-in=androidx.compose.material3.ExperimentalMaterial3ExpressiveApi",
        )
    }
}

