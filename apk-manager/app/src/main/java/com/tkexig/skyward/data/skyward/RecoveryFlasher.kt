package com.tkexig.skyward.data.skyward

import android.util.Log
import com.topjohnwu.superuser.ShellUtils
import com.tkexig.skyward.ui.util.getRootShell
import java.io.File

/**
 * 把一份 recovery 镜像写进手机的分区。
 *
 * 只做一件事：`dd` 一个 img 到 recovery 分区。Skyward 自己的安装器要处理的
 * 是 OTA 包、VAB 逻辑分区、payload 差分包那些，那些东西在这里用不上——用户
 * 要的是把手上这份镜像刷进去，然后重启到 recovery。
 *
 * 写之前一定先备份：写错分区或者镜像不对，手机就进不去 recovery 了，那时
 * 手上得有原来那份才能刷回去。
 */
object RecoveryFlasher {
    private const val TAG = "RecoveryFlasher"

    /** 分区设备路径。A/B 机器上 recovery 通常合在 boot 里，没有单独这个分区。 */
    private fun partitionDevice(): String = "/dev/block/by-name/recovery"

    /** 这个分区在不在。 */
    fun hasRecoveryPartition(): Boolean {
        val out = ShellUtils.fastCmd(
            getRootShell(globalMnt = true),
            "[ -e ${partitionDevice()} ] && echo yes || echo no"
        )
        return out.trim() == "yes"
    }

    /**
     * 当前槽位，形如 `_a`。
     *
     * 非 A/B 机器上是空的。取不到就当空，不猜。
     */
    fun currentSlotSuffix(): String {
        val slot = ShellUtils.fastCmd(
            getRootShell(globalMnt = true),
            "getprop ro.boot.slot_suffix"
        ).trim()
        return slot
    }

    /**
     * 备份当前 recovery 到应用私有目录。
     *
     * 存到 filesDir 而不是外部存储：这是救命用的文件，不该被清理工具扫走，
     * 也不需要存储权限。
     */
    fun backupCurrent(destDir: File): Result<File> = runCatching {
        if (!hasRecoveryPartition()) error("这台机器没有独立的 recovery 分区（可能是 A/B 机型）")
        destDir.mkdirs()
        val stamp = java.text.SimpleDateFormat("yyyyMMdd-HHmmss", java.util.Locale.US)
            .format(java.util.Date())
        val dest = File(destDir, "recovery-backup-$stamp.img")
        val out = ShellUtils.fastCmd(
            getRootShell(globalMnt = true),
            "dd if=${partitionDevice()} of=${dest.absolutePath.shellQuote()} bs=4M 2>&1 && echo ok"
        ).trim()
        if (!out.contains("ok")) error("备份失败：${out.takeLast(200)}")
        if (!dest.isFile || dest.length() == 0L) error("备份文件是空的")
        dest
    }

    /**
     * 把 [image] 写进 recovery 分区。
     *
     * `dd` 到 by-name 设备而不是挂载点：分区没有文件系统，直接按字节写。
     * 写完 sync 再回话，不然断电时可能只写了一半。
     */
    fun flash(image: File): Result<Unit> = runCatching {
        if (!image.isFile) error("找不到要刷入的文件：${image.absolutePath}")
        if (image.length() == 0L) error("文件是空的")
        if (!hasRecoveryPartition()) error("这台机器没有独立的 recovery 分区（可能是 A/B 机型）")

        Log.i(TAG, "flashing ${image.absolutePath} (${image.length()} bytes) -> ${partitionDevice()}")
        val out = ShellUtils.fastCmd(
            getRootShell(globalMnt = true),
            "dd if=${image.absolutePath.shellQuote()} of=${partitionDevice()} bs=4M 2>&1 && sync && echo ok"
        ).trim()
        if (!out.contains("ok")) error("刷入失败：${out.takeLast(200)}")
    }

    /**
     * 从一个 Skyward 刷机包里取出 recovery 镜像。
     *
     * 官方包是 zip。里面可能是 recovery.img，也可能只有 payload.bin（OTA
     * 那种），后者这里解不了——遇到就明确说不支持，而不是写个空文件进去。
     */
    fun extractImageFromZip(zip: File, destDir: File): Result<File> = runCatching {
        if (!zip.isFile) error("找不到刷机包：${zip.absolutePath}")
        destDir.mkdirs()
        val out = File(destDir, "recovery.img")
        // 用系统的 unzip：它一定在，而且不用为了解一个文件引入依赖。
        val listing = ShellUtils.fastCmd(
            getRootShell(globalMnt = true),
            "unzip -l ${zip.absolutePath.shellQuote()} 2>&1"
        )
        val entry = listOf("recovery.img", "recovery-*.img", "*.img")
            .firstNotNullOfOrNull { pattern ->
                listing.lineSequence()
                    .map { it.trim().substringAfterLast(' ') }
                    .firstOrNull { name ->
                        name.isNotBlank() && name.endsWith(".img") &&
                            (pattern == "*.img" || name == pattern || name.startsWith("recovery-"))
                    }
            } ?: error("这个包里没有 .img 镜像（可能是 OTA 包，这里刷不了）")

        val unzip = ShellUtils.fastCmd(
            getRootShell(globalMnt = true),
            "unzip -o -p ${zip.absolutePath.shellQuote()} ${entry.shellQuote()} > " +
                "${out.absolutePath.shellQuote()} && echo ok"
        ).trim()
        if (!unzip.contains("ok")) error("从包里取出镜像失败：${unzip.takeLast(200)}")
        if (!out.isFile || out.length() == 0L) error("取出来的镜像是空的")
        out
    }
}
