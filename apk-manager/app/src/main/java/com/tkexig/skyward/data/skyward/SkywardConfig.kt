package com.tkexig.skyward.data.skyward

import android.util.Log
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
object SkywardConfig {
    private const val TAG = "SkywardConfig"

    const val DIR = "/metadata/skyward"
    const val FILE = "$DIR/settings.json"

    /** /metadata 挂上了没有。 */
    private fun mounted(): Boolean =
        ShellUtils.fastCmd(getRootShell(), "mountpoint -q /metadata && echo yes || echo no")
            .trim() == "yes"

    /**
     * 让 /metadata 可读：没挂就挂上。
     *
     * `metadata` 这个 fstab 名字是 Skyward 自己用的；挂不上就返回 false，调用
     * 方据此报「读不到」，而不是编一份默认配置出来覆盖用户的。
     */
    private fun ensureMounted(): Boolean {
        if (mounted()) return true
        val out = ShellUtils.fastCmd(getRootShell(), "mount /metadata 2>&1")
        if (mounted()) return true
        Log.w(TAG, "mount /metadata: $out")
        return false
    }

    /**
     * 读配置。
     *
     * 返回 null 表示「确实没有配置」或「读不出来」——两种情况调用方都该显示
     * 默认值，但不能因此把文件写掉：一份读不出来的配置可能是新版 Skyward 写
     * 的，覆盖了就丢了。
     */
    fun read(): SkywardSettings? {
        if (!ensureMounted()) return null
        val text = runCatching {
            ShellUtils.fastCmd(getRootShell(), "cat ${FILE.shellQuote()}")
        }.getOrElse {
            Log.w(TAG, "cat $FILE", it)
            return null
        }
        if (text.isBlank()) return null
        return runCatching { SkywardSettings.json.decodeFromString<SkywardSettings>(text) }
            .onFailure { Log.w(TAG, "$FILE does not read as settings", it) }
            .getOrNull()
    }

    /**
     * 写配置：先写旁边的临时文件，再整体移过去。
     *
     * 和 Skyward 自己写的方式一致——写到一半断电，留下的也是旧配置，不是半
     * 份新的。
     */
    fun write(settings: SkywardSettings): Result<Unit> = runCatching {
        if (!ensureMounted()) error("/metadata 挂不上，配置写不进去")
        val text = SkywardSettings.json.encodeToString(SkywardSettings.serializer(), settings)
        val tmp = "$FILE.new"
        // base64 传内容：配置里有中文和换行，直接拼进 shell 会被拆开。
        val b64 = android.util.Base64.encodeToString(text.toByteArray(), android.util.Base64.NO_WRAP)
        val out = ShellUtils.fastCmd(
            getRootShell(),
            "mkdir -p $DIR && echo $b64 | base64 -d > ${tmp.shellQuote()} && " +
                "chmod 600 ${tmp.shellQuote()} && mv -f ${tmp.shellQuote()} ${FILE.shellQuote()} && echo ok"
        ).trim()
        if (out != "ok") error("写 $FILE 失败：$out")
    }

    /** 配置文件在不在（挂上 /metadata 之后看）。 */
    fun exists(): Boolean {
        if (!ensureMounted()) return false
        val out = ShellUtils.fastCmd(getRootShell(), "[ -f ${FILE.shellQuote()} ] && echo yes || echo no")
        return out.trim() == "yes"
    }

    /**
     * 把一份配置导出成 JSON 文本，给「备份」用。
     *
     * 备份成 JSON 而不是二进制：这份文件本来就是 JSON，人看得懂、能改，换台
     * 机器也能用。
     */
    fun exportJson(): String? = read()?.let {
        SkywardSettings.json.encodeToString(SkywardSettings.serializer(), it)
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
