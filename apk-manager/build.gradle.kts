plugins {
    alias(libs.plugins.agp.app) apply false
    alias(libs.plugins.agp.library) apply false
    alias(libs.plugins.kotlin) apply false
    alias(libs.plugins.compose.compiler) apply false
}

extra["androidMinSdkVersion"] = 24
extra["androidTargetSdkVersion"] = 37
extra["androidCompileSdkVersion"] = 37
extra["androidCompileSdkVersionMinor"] = 0
extra["androidBuildToolsVersion"] = "37.0.0"
extra["androidSourceCompatibility"] = JavaVersion.VERSION_21
extra["androidTargetCompatibility"] = JavaVersion.VERSION_21
// The version lives in version.properties, not in git tags: every build bumps
// the patch number by 0.0.1, and CI commits the bump back. The APK name and the
// "new version" prompt both read it from here.
val versionFile = file("version.properties")

fun readVersion(): String {
    val props = java.util.Properties()
    if (versionFile.isFile) versionFile.inputStream().use { props.load(it) }
    return props.getProperty("versionName", "1.0.0").trim()
}

/**
 * 1.2.3 -> 1 * 10000 + 2 * 100 + 3. Android only orders versionCode, so the
 * name's three parts have to fit into one number; each part gets two digits,
 * which is what the bump by 0.0.1 relies on.
 */
fun versionCodeOf(name: String): Int {
    val parts = name.split(".").map { it.toIntOrNull() ?: 0 }
    val (major, minor, patch) = Triple(
        parts.getOrElse(0) { 0 },
        parts.getOrElse(1) { 0 },
        parts.getOrElse(2) { 0 },
    )
    return major * 10000 + minor * 100 + patch
}

extra["managerVersionName"] = readVersion()
extra["managerVersionCode"] = versionCodeOf(readVersion())