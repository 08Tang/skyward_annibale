package com.tkexig.skyward.data.skyward

import android.util.Log
import com.topjohnwu.superuser.ShellUtils
import com.tkexig.skyward.ui.util.getRootShell
import org.json.JSONObject

/**
 * 哪些应用被授权了 root。
 *
 * SukiSU 把这份名单存在内核模块里，由 supercall 读写——这是它作为「Root
 * 管理器」的核心特征，也是这个 APK 要丢掉的东西。这里改成一份自己维护的
 * JSON，放在 APK 自己的数据目录里。
 *
 * 注意：这只是一份**记录**，不是授权本身。真正的授权由系统那个 su（比如
 * Magisk 或 KernelSU 的 su）在它自己的界面里完成；这里记的是「我看到它拿到
 * 了 root」，用来给首页和超级用户页显示。
 */
object SuGrants {
    private const val TAG = "SuGrants"

    /** 名单文件：APK 自己的私有目录，不需要 root 就能读。 */
    private fun file() = java.io.File(
        com.tkexig.skyward.ksuApp.filesDir,
        "su_grants.json"
    )

    private fun load(): MutableMap<String, Boolean> = runCatching {
        val f = file()
        if (!f.isFile) return mutableMapOf()
        val obj = JSONObject(f.readText())
        buildMap {
            obj.keys().forEach { key -> put(key, obj.optBoolean(key, false)) }
        }.toMutableMap()
    }.getOrElse {
        Log.w(TAG, "the grant list does not read", it)
        mutableMapOf()
    }

    private fun save(grants: Map<String, Boolean>): Boolean = runCatching {
        val obj = JSONObject()
        grants.forEach { (k, v) -> obj.put(k, v) }
        file().writeText(obj.toString())
        true
    }.getOrElse {
        Log.w(TAG, "the grant list does not write", it)
        false
    }

    fun isGranted(key: String): Boolean = load()[key] == true

    fun set(key: String, granted: Boolean): Boolean {
        if (key.isBlank()) return false
        val grants = load()
        if (granted) grants[key] = true else grants.remove(key)
        return save(grants)
    }

    /** 授权了几个。 */
    fun count(): Int = load().count { it.value }

    /** 全部授权的包名，给超级用户页列出来。 */
    fun granted(): List<String> = load().filterValues { it }.keys.toList()

    /**
     * 这个 uid 的用户名。
     *
     * /proc 里没有 uid 到包名的映射，只有 /data/system/packages.list 有；读
     * 不到就退回 uid 本身，页面照样显示得出东西。
     */
    fun userName(uid: Int): String? {
        if (uid < 0) return null
        val out = runCatching {
            ShellUtils.fastCmd(
                getRootShell(),
                "awk -v u=$uid '\$2==u{print \$1; exit}' /data/system/packages.list 2>/dev/null"
            )
        }.getOrNull()?.trim().orEmpty()
        return out.ifEmpty { uid.toString() }
    }
}
