package com.tkexig.skyward.ui.util

import android.content.Context
import android.os.Build
import android.system.Os
import com.tkexig.skyward.BuildConfig
import com.tkexig.skyward.data.log.AppLog
import java.io.File
import java.io.FileWriter
import java.io.PrintWriter
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

/**
 * 出问题时导出的那一份东西。
 *
 * 只放这个应用自己的日志和一点设备信息。原来是 SukiSU 的 bugreport：它会去
 * 抓 `/data/adb/ksu` 的内容、授权名单、kallsyms、内核模块列表——那些东西既
 * 不是这个应用产生的，也不是排查它需要看的，还正好是 Root 管理器会碰的
 * 位置。这里不碰。
 *
 * 不需要 root：全是应用自己目录里的文件加上 Build 常量。
 */
fun getBugreportFile(context: Context): File {
    val stamp = LocalDateTime.now().format(DateTimeFormatter.ofPattern("yyyy-MM-dd_HH_mm"))
    val target = File(context.cacheDir, "Skyward-log-$stamp.txt")

    PrintWriter(FileWriter(target)).use { out ->
        out.println("Skyward Manager ${BuildConfig.VERSION_NAME} (${BuildConfig.VERSION_CODE})")
        out.println("Time: ${LocalDateTime.now()}")
        out.println()
        out.println("Device")
        out.println("  brand: ${Build.BRAND}")
        out.println("  model: ${Build.MODEL}")
        out.println("  product: ${Build.PRODUCT}")
        out.println("  manufacturer: ${Build.MANUFACTURER}")
        out.println("  device: ${Build.DEVICE}")
        out.println("  sdk: ${Build.VERSION.SDK_INT}")
        out.println("  fingerprint: ${Build.FINGERPRINT}")
        out.println("  abis: ${Build.SUPPORTED_ABIS.joinToString()}")
        runCatching {
            val uname = Os.uname()
            out.println("  kernel: ${uname.release} (${uname.version})")
        }
        out.println()
        out.println("App log")
        out.println("-------")
        val log = AppLog.read()
        out.println(log.ifBlank { "(empty)" })
    }
    return target
}
