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

    /**
     * 分区设备路径。
     *
     * A/B 机器上分区名带槽位后缀（`recovery_a`、`recovery_b`），不带后缀的那
     * 个往往只是指向当前槽的符号链接；要刷另一槽就得指名道姓。非 A/B 机器
     * 只有 `recovery`。
     */
    private fun partitionDevice(slot: String = ""): String {
        val base = "/dev/block/by-name/recovery"
        if (slot.isBlank()) return base
        val slotted = "$base$slot"
        // 设备节点在应用自己的命名空间里看不到，要用 shell 判。
        val exists = ShellUtils.fastCmd(
            getRootShell(globalMnt = true), "[ -e $slotted ] && echo y || echo n"
        ).trim() == "y"
        return if (exists) slotted else base
    }

    /**
     * A/B 机器上的两个槽。
     *
     * 靠 `ro.boot.slot_suffix` 有没有值判断是不是 A/B：有的话就是 `_a`/`_b`
     * 两个，没有就返回空表，界面上不给选。
     */
    fun availableSlots(): List<String> {
        val current = currentSlotSuffix()
        if (current.isBlank()) return emptyList()
        return listOf("_a", "_b")
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
    fun backupCurrent(destDir: File, slot: String = ""): Result<File> = runCatching {
        destDir.mkdirs()
        val stamp = java.text.SimpleDateFormat("yyyyMMdd-HHmmss", java.util.Locale.US)
            .format(java.util.Date())
        val dest = File(destDir, "recovery-backup-$stamp.img")
        val out = ShellUtils.fastCmd(
            getRootShell(globalMnt = true),
            "dd if=${partitionDevice(slot)} of=${dest.absolutePath.shellQuote()} bs=4M 2>&1 && echo ok"
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
    fun flash(image: File, slot: String = ""): Result<Unit> = runCatching {
        if (!image.isFile) error("找不到要刷入的文件：${image.absolutePath}")
        if (image.length() == 0L) error("文件是空的")

        val device = partitionDevice(slot)
        Log.i(TAG, "flashing ${image.absolutePath} (${image.length()} bytes) -> $device")
        val out = ShellUtils.fastCmd(
            getRootShell(globalMnt = true),
            "dd if=${image.absolutePath.shellQuote()} of=$device bs=4M 2>&1 && sync && echo ok"
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
