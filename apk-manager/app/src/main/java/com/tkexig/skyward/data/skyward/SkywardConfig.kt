package com.tkexig.skyward.data.skyward

import com.tkexig.skyward.data.log.AppLog
import com.topjohnwu.superuser.ShellUtils
import com.tkexig.skyward.ui.util.getRootShell
import java.io.File

/**
 * Skyward 的配置放在 /metadata/skyward/settings.json。
 *
 * 为什么不能直接 File().readText()：Skyward 平时让 /metadata 只读，写的时候
 * 才短暂放开（`skyward_vab::metadata_writable`）。系统启动完成后 /metadata
 * 通常根本没挂载，路径是个空目录，直接读会读到「没有配置」，而实际配置还
 * 在分区里。所以读写都先确保挂载，再动文件。
 */
/**
 * 读配置的结果。
 *
 * 分成三种而不是「读到或 null」：没有文件该显示默认值并说明「还没有配置」，
 * 读失败该明确报错——两种情况都显示默认值的话，用户看到的就是一堆默认值
 * 配着「一切正常」，那正是之前灭屏和温控看着不对的样子。
 */
sealed interface SkywardRead {
    /** 读到了。 */
    data class Loaded(val settings: SkywardSettings) : SkywardRead
    /** 文件不存在，用 recovery 的默认值。 */
    data object Missing : SkywardRead
    /** 文件在但读不出来。 */
    data class Failed(val reason: String) : SkywardRead
}

object SkywardConfig {
    const val DIR = "/metadata/skyward"
    const val FILE = "$DIR/settings.json"

    /**
     * /metadata 挂上了没有。
     *
     * 看 /proc/mounts，不用 `mountpoint`：那个命令不是每台机器的 toybox 里
     * 都有，没有的时候它报的错和「没挂」一模一样，于是这里以为没挂、去挂
     * 一个已经挂着的挂载点、挂不上，最后整份配置读不到、界面全显示默认值。
     */
    private fun mounted(): Boolean {
        val yes = ShellUtils.fastCmd(
            getRootShell(globalMnt = true),
            "grep -qs ' /metadata ' /proc/mounts && echo yes || echo no"
        ).trim() == "yes"
        AppLog.d("mounted(/metadata) = $yes")
        return yes
    }

    /**
     * 让 /metadata 可读：没挂就挂上。
     *
     * 不能用 `mount /metadata`：那是按 fstab 里的条目挂，而 fstab 是 recovery
     * 自己的，系统跑起来之后 /metadata 往往根本没有这一条。Skyward 自己是按
     * 分区设备挂的（`/dev/block/by-name/metadata`），文件系统依次试 erofs、
     * ext4、f2fs——这里照它的顺序试，因为同一台机器上分区是什么格式要看
     * 厂商，猜错就是挂不上。
     *
     * 挂不上就返回 false，调用方据此报「读不到」，而不是编一份默认配置出来
     * 覆盖用户的。
     */
    private fun ensureMounted(): Boolean {
        if (mounted()) return true
        val dev = "/dev/block/by-name/metadata"
        AppLog.i("挂载 /metadata：试 $dev")
        for (fs in listOf("erofs", "ext4", "f2fs")) {
            val out = ShellUtils.fastCmd(
                getRootShell(globalMnt = true),
                "mkdir -p /metadata; mount -t $fs -o ro $dev /metadata 2>&1"
            )
            AppLog.d("mount -t $fs → ${out.ifBlank { "(无输出)" }}")
            if (mounted()) {
                AppLog.i("/metadata 挂上了（$fs）")
                return true
            }
        }
        AppLog.e("cannot mount $dev on /metadata as erofs, ext4 or f2fs")
        return false
    }

    /**
     * 读配置。
     *
     * 返回 null 表示「确实没有配置」或「读不出来」——两种情况调用方都该显示
     * 默认值，但不能因此把文件写掉：一份读不出来的配置可能是新版 Skyward 写
     * 的，覆盖了就丢了。
     */
    fun read(): SkywardRead {
        if (!ensureMounted()) return SkywardRead.Failed("/metadata 挂不上，读不到配置")
        // base64 传回来再解开，不直接 cat：libsu 按行给结果，而这份 JSON 是
        // 多行格式化的，拼行会丢掉行尾的空格、也容易在换行处出错。base64 是
        // 一整行，怎么读都不会坏。
        var b64Chars = 0
        val text = runCatching {
            // -w0 让 base64 不折行；toybox 的老版本不认这个参数，那就退回
            // 不带它，折出来的行下面会拼回去。
            val shell = getRootShell(globalMnt = true)
            val quoted = FILE.shellQuote()
            var out = shell.newJob().add("base64 -w0 $quoted 2>/dev/null").exec().out
            if (out.isEmpty()) out = shell.newJob().add("base64 $quoted").exec().out
            val b64 = out.joinToString("").filterNot { it.isWhitespace() }
            b64Chars = b64.length
            if (b64.isBlank()) "" else String(
                android.util.Base64.decode(b64, android.util.Base64.DEFAULT),
                Charsets.UTF_8,
            )
        }.getOrElse {
            AppLog.e("读 $FILE 失败", it)
            return SkywardRead.Failed("读 $FILE 出错：${it.message}")
        }
        if (text.isBlank()) {
            AppLog.w("$FILE 是空的（base64 解出 0 字节）")
            return SkywardRead.Missing
        }
        AppLog.i("读到 $FILE：${text.length} 字符，${text.lines().size} 行，$b64Chars 个 base64 字符")
        AppLog.d("开头：${text.take(120).replace("\n", "\\n")}")
        AppLog.d("结尾：${text.takeLast(60).replace("\n", "\\n")}")
        return runCatching { SkywardSettings.json.decodeFromString<SkywardSettings>(text) }
            .fold(
                onSuccess = {
                    AppLog.i(
                        "解码成功：screen_off=${it.screenOff}s thermal_limit=${it.thermalLimit}" +
                            " charge_mode=${it.chargeMode} brightness=${it.brightness}"
                    )
                    SkywardRead.Loaded(it)
                },
                onFailure = {
                    AppLog.e("$FILE 解码失败", it)
                    SkywardRead.Failed("配置文件在，但读不懂：${it.message}")
                },
            )
    }

    /**
     * 写配置：先写旁边的临时文件，再整体移过去。
     *
     * 和 Skyward 自己写的方式一致——写到一半断电，留下的也是旧配置，不是半
     * 份新的。
     */
    fun write(settings: SkywardSettings): Result<Unit> = runCatching {
        if (!ensureMounted()) error("/metadata 挂不上，配置写不进去")
        // 读的时候是只读挂的，写之前要重挂成可写；remount 不动里面的内容。
        val remount = ShellUtils.fastCmd(
            getRootShell(globalMnt = true),
            "mount -o remount,rw /metadata 2>&1 && echo ok"
        ).trim()
        AppLog.d("remount rw → ${remount.ifBlank { "(无输出)" }}")
        if (remount != "ok") error("/metadata 挂不成可写：$remount")
        val text = SkywardSettings.json.encodeToString(SkywardSettings.serializer(), settings)
        AppLog.i("写入 $FILE：${text.length} 字符，screen_off=${settings.screenOff} thermal_limit=${settings.thermalLimit}")
        val tmp = "$FILE.new"
        // 内容走 stdin，不拼进命令行：base64 有上千字符，拼进去会撞上命令行
        // 长度上限，而且配置里的引号、换行都要另想办法转义。
        val b64 = android.util.Base64.encodeToString(text.toByteArray(), android.util.Base64.NO_WRAP)
        val out = getRootShell(globalMnt = true)
            .newJob()
            .add("mkdir -p $DIR && base64 -d > ${tmp.shellQuote()} && " +
                "chmod 600 ${tmp.shellQuote()} && mv -f ${tmp.shellQuote()} ${FILE.shellQuote()} && echo ok")
            .to(ArrayList(), listOf(b64))
            .exec()
        val said = out.out.joinToString(" ").trim()
        if (!out.isSuccess || !said.contains("ok")) {
            AppLog.e("写 $FILE 失败：${said.ifBlank { out.err.joinToString(" ") }}")
            error("写 $FILE 失败：${said.ifBlank { "无输出" }}")
        }
        AppLog.i("写入完成")
    }

    /** 配置文件在不在（挂上 /metadata 之后看）。 */
    fun exists(): Boolean {
        if (!ensureMounted()) return false
        val out = ShellUtils.fastCmd(getRootShell(globalMnt = true), "[ -f ${FILE.shellQuote()} ] && echo yes || echo no")
        return out.trim() == "yes"
    }

    /**
     * 把一份配置导出成 JSON 文本，给「备份」用。
     *
     * 备份成 JSON 而不是二进制：这份文件本来就是 JSON，人看得懂、能改，换台
     * 机器也能用。
     */
    fun exportJson(): String? = (read() as? SkywardRead.Loaded)?.let {
        SkywardSettings.json.encodeToString(SkywardSettings.serializer(), it.settings)
    }

    /** 从 JSON 文本导入；解析不了就返回失败，不覆盖现有配置。 */
    fun importJson(text: String): Result<SkywardSettings> = runCatching {
        val settings = SkywardSettings.json.decodeFromString<SkywardSettings>(text)
        write(settings).getOrThrow()
        settings
    }
}

/** Shell 单引号，和 Skyward 那边的 shellQuote 一个意思。 */
internal fun String.shellQuote(): String = "'" + replace("'", "'\\''") + "'"
