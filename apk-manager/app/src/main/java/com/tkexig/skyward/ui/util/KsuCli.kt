package com.tkexig.skyward.ui.util

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import android.system.Os
import android.util.Log
import com.topjohnwu.superuser.Shell
import com.topjohnwu.superuser.ShellUtils
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import com.tkexig.skyward.BuildConfig
import com.tkexig.skyward.ksuApp
import java.io.File

/**
 * 跑 root 命令的那一层。
 *
 * 原来这个文件是 SukiSU 的 ksud 包装：模块安装、boot 修补、KPM、SUSFS、
 * 内置 zygisk、卸载管理、扩展……上百个函数，每个都是一句
 * `libksud.so xxx`。这一版不要 ksud（它要和内核模块配对，正是要去掉的
 * 特征），所以只剩纯 su 能做的事：开 shell、重启、读设备信息、看有没有
 * root。其余能力在对应的页面里已经删掉了。
 */
private const val TAG = "KsuCli"

// Single-quote a value for safe interpolation
fun String.shellQuote(): String = "'" + replace("'", "'\\''") + "'"

object KsuCli {
    val SHELL: Shell = createRootShell()
    val GLOBAL_MNT_SHELL: Shell = createRootShell(true)
}

fun getRootShell(globalMnt: Boolean = false): Shell {
    return if (globalMnt) KsuCli.GLOBAL_MNT_SHELL else {
        KsuCli.SHELL
    }
}

inline fun <T> withNewRootShell(
    globalMnt: Boolean = false,
    block: Shell.() -> T
): T {
    return createRootShell(globalMnt).use(block)
}

fun Uri.getFileName(context: Context): String? = runCatching {
    context.contentResolver.query(
        this,
        arrayOf(OpenableColumns.DISPLAY_NAME),
        null,
        null,
        null,
    )?.use { cursor ->
        val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
        if (index >= 0 && cursor.moveToFirst()) cursor.getString(index) else null
    }
}.getOrNull()

/**
 * 拿一个 root shell。
 *
 * 只用系统那个 su。原来这里是「先试 APK 里打包的 ksud，失败再退回 su」——
 * ksud 是 SukiSU 自己的守护进程，要和内核模块配对，正是这个 APK 要去掉的
 * 特征。现在没有内核模块，也就没有 ksud，直接问 su。
 *
 * su 也没有的话退回 sh：页面还能打开看，只是每件事都会报没有 root。
 */
fun createRootShell(globalMnt: Boolean = false): Shell {
    Shell.enableVerboseLogging = BuildConfig.DEBUG
    val builder = Shell.Builder.create()
    return try {
        // -mm 是 Magisk 的「全局挂载命名空间」，改 /metadata 时要它。
        if (globalMnt) builder.build("su", "-mm") else builder.build("su")
    } catch (e: Throwable) {
        Log.e(TAG, "su failed: ", e)
        builder.build("sh")
    }
}

/**
 * ksud 的替身。
 *
 * SukiSU 里那些开关（sulog、adb_root、selinux_hide 之类）都是内核模块的
 * feature，由 ksud 这个守护进程读写。这一版没有内核模块，也没有 ksud，
 * 所以这里不假装成功：一律返回 false，调用方会走「没生效」那条路。
 */
fun execKsud(args: String, newShell: Boolean = false, globalMnt: Boolean = false): Boolean = false

suspend fun getFeatureStatus(feature: String): String = ""

suspend fun getFeaturePersistValue(feature: String): Long? = null

// Current value in kernel
suspend fun getFeatureValue(feature: String): Long? = null

fun reboot(reason: String = "") {
    val shell = getRootShell()
    if (reason == "recovery") {
        // KEYCODE_POWER = 26, hide incorrect "Factory data reset" message
        ShellUtils.fastCmd(shell, "/system/bin/input keyevent 26")
    }
    ShellUtils.fastCmd(shell, "/system/bin/svc power reboot $reason || /system/bin/reboot $reason")
}

fun rootAvailable(): Boolean {
    val shell = getRootShell()
    return shell.isRoot
}

suspend fun runCmd(cmd: String): List<String> = withContext(Dispatchers.IO) {
    val shell = getRootShell()
    shell.newJob().add(cmd).to(ArrayList(), null).exec().out
}

/**
 * 当前内核的 KMI，形如 `android12-5.10`。
 *
 * ksud 有个 `boot-info current-kmi` 专门报这个，这里按内核版本自己拼：
 * Android 主版本从 `Os.uname().release` 里取不到，就从 ro.build.version
 * 里读，取不到就只报内核版本。
 */
suspend fun getCurrentKmi(): String = withContext(Dispatchers.IO) {
    val shell = getRootShell()
    val android = ShellUtils.fastCmd(shell, "getprop ro.build.version.release").trim()
        .substringBefore('.').toIntOrNull()
    val release = Os.uname().release
    if (android != null) "android$android-$release" else release
}

/**
 * 支持哪些 KMI。
 *
 * 原来这是问 ksud 内核支持哪些 KMI 以便下载对应 boot 镜像。这一版不刷
 * boot，也就没有「支持列表」可问，返回当前这一个。
 */
suspend fun getSupportedKmis(): List<String> = listOf(getCurrentKmi())

suspend fun isAbDevice(): Boolean = withContext(Dispatchers.IO) {
    val shell = getRootShell()
    val slot = ShellUtils.fastCmd(shell, "getprop ro.boot.slot_suffix").trim()
    slot.isNotEmpty()
}

suspend fun getDefaultPartition(): String = withContext(Dispatchers.IO) {
    if (!Os.uname().release.contains("android12-")) "init_boot" else "boot"
}

suspend fun getSlotSuffix(ota: Boolean): String = withContext(Dispatchers.IO) {
    val shell = getRootShell()
    ShellUtils.fastCmd(shell, "getprop ro.boot.slot_suffix").trim()
}

suspend fun getAvailablePartitions(): List<String> = withContext(Dispatchers.IO) {
    listOf("boot", "init_boot", "vendor_boot")
}

fun hasMagisk(): Boolean {
    val shell = getRootShell(true)
    val result = shell.newJob().add("which magisk").exec()
    Log.i(TAG, "has magisk: ${result.isSuccess}")
    return result.isSuccess
}

/**
 * sepolicy 规则是否合法。
 *
 * SukiSU 是把规则丢给 ksud 让内核去校验的。这一版没有内核 sepolicy 可改，
 * 所以只做基本检查：非空、每行不是乱码。真正的生效与否不归这个 APK 管。
 */
fun isSepolicyValid(rules: String?): Boolean {
    if (rules == null) return true
    if (rules.isBlank()) return true
    return rules.lineSequence()
        .map { it.trim() }
        .filter { it.isNotEmpty() && !it.startsWith("#") }
        .all { it.contains(' ') }
}

fun forceStopApp(pkg: String) {
    val shell = getRootShell()
    ShellUtils.fastCmd(shell, "am force-stop ${pkg.shellQuote()}")
}

fun launchApp(pkg: String) {
    val shell = getRootShell()
    ShellUtils.fastCmd(shell, "monkey -p ${pkg.shellQuote()} -c android.intent.category.LAUNCHER 1")
}

fun restartApp(pkg: String) {
    forceStopApp(pkg)
    launchApp(pkg)
}

fun removeAsRoot(path: String): Boolean {
    val shell = getRootShell()
    return shell.newJob().add("rm -rf ${path.shellQuote()}").exec().isSuccess
}
