package com.tkexig.skyward

import android.os.Parcelable
import androidx.annotation.Keep
import androidx.compose.runtime.Immutable
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import com.tkexig.skyward.Natives.Profile.RootProfileFlag
import com.tkexig.skyward.ui.util.rootAvailable
import com.tkexig.skyward.data.skyward.SuGrants

/**
 * @author weishu
 * @date 2022/12/8.
 */
object Natives {
    // minimal supported kernel version
    // 10915: allowlist breaking change, add app profile
    // 10931: app profile struct add 'version' field
    // 10946: add capabilities
    // 10977: change groups_count and groups to avoid overflow write
    // 11071: Fix the issue of failing to set a custom SELinux type.
    // 12143: breaking: new supercall impl
    // 32310: new get_allow_list ioctl
    // 32336: new set_sepolicy ioctl
    // 32377: add set_init_pgrp ioctl
    // 32513: add uapi version
    const val MINIMAL_SUPPORTED_KERNEL = 32513

    // 没有内核模块，就没有可读的完整版本。
    fun getFullVersion(): String? = null
    const val MINIMAL_SUPPORTED_KERNEL_FULL = "v3.0.0"

    // 12040: Support disable sucompat mode
    const val KERNEL_SU_DOMAIN = "u:r:ksu:s0"

    const val ROOT_UID = 0
    const val ROOT_GID = 0

    fun isVersionLessThan(v1Full: String, v2Full: String): Boolean {
        fun extractVersionParts(version: String): List<Int> {
            val match = Regex("""v\d+(\.\d+)*""").find(version)
            val simpleVersion = match?.value ?: version
            return simpleVersion.trimStart('v').split('.').map { it.toIntOrNull() ?: 0 }
        }

        val v1Parts = extractVersionParts(v1Full)
        val v2Parts = extractVersionParts(v2Full)
        val maxLength = maxOf(v1Parts.size, v2Parts.size)
        for (i in 0 until maxLength) {
            val num1 = v1Parts.getOrElse(i) { 0 }
            val num2 = v2Parts.getOrElse(i) { 0 }
            if (num1 != num2) return num1 < num2
        }
        return false
    }

    /**
     * 这一版不再和内核里的 SukiSU 模块说话：root 完全走系统的 su。
     *
     * 下面这些原本都是 supercall / ioctl，是 Root 管理器的特征——内核要装
     * 一个自己的模块，APK 再通过它读授权名单、隐藏 SELinux、改挂载。这里
     * 全部换成本机就能做到的事：读 /proc、跑 su、读 Skyward 的配置。
     */

    /** 没有 SukiSU 内核，版本号报 0，页面据此走「无内核」的分支。 */
    val version: Int
        get() = 0

    /** 安全模式是内核模块的概念；这里没有。 */
    val isSafeMode: Boolean
        get() = false

    val isLkmMode: Boolean
        get() = false

    val isLkmBundled: Boolean
        get() = false

    val isLateLoadMode: Boolean
        get() = false

    /**
     * 「这个应用现在能用吗」。
     *
     * 在 SukiSU 里它问的是「APK 和内核里的模块配对了吗」；这一版没有内核
     * 模块，配对这件事不存在，能拿到 root 就是能用。首页那张卡片的「工作中」
     * 就是它。
     */
    val isManager: Boolean
        get() = rootAvailable()

    val isPrBuild: Boolean
        get() = BuildConfig.IS_PR_BUILD

    /** 没有按 uid 卸载模块这回事：模块由 Skyward 自己装。 */
    fun uidShouldUmount(uid: Int): Boolean = false

    /**
     * 授权名单存在 Skyward 的配置里，不存内核。
     *
     * 这里只回答「这个包有没有被授权」，读的是 APK 自己维护的一份 JSON；
     * 没有装 SukiSU 内核，内核也不会替我们记。
     */
    fun getAppProfile(key: String?, uid: Int): Profile {
        val name = key ?: return Profile("")
        val granted = SuGrants.isGranted(name)
        return Profile(name, uid, allowSu = granted)
    }

    fun setAppProfile(profile: Profile?): Boolean {
        val p = profile ?: return false
        return SuGrants.set(p.name, p.allowSu)
    }

    fun isSuEnabled(): Boolean = true

    fun setSuEnabled(enabled: Boolean): Boolean = true

    fun isKernelUmountEnabled(): Boolean = true

    fun setKernelUmountEnabled(enabled: Boolean): Boolean = true

    /** SELinux 隐藏是内核模块做的；这里没有内核模块，也就没有可隐藏的。 */
    fun isSelinuxHideEnabled(): Boolean = false

    fun setSelinuxHideEnabled(enabled: Boolean): Int = -1

    fun isAvcSpoofEnabled(): Boolean = false

    fun setAvcSpoofEnabled(enabled: Boolean): Boolean = false

    fun isMountHideEnabled(): Boolean = false

    fun setMountHideEnabled(enabled: Boolean): Int = -1

    /** /proc 里就有用户名，不必问内核。 */
    fun getUserName(uid: Int): String? = SuGrants.userName(uid)

    /** 授权了几个应用：数自己的名单。 */
    fun getSuperuserCount(): Int = SuGrants.count()

    /** 没有 hook，就没有 hook 的类型可报。 */
    fun getHookType(): String = ""

    private const val NON_ROOT_DEFAULT_PROFILE_KEY = "$"
    private const val NOBODY_UID = 9999

    fun setDefaultUmountModules(umountModules: Boolean): Boolean {
        Profile(
            NON_ROOT_DEFAULT_PROFILE_KEY,
            NOBODY_UID,
            false,
            umountModules = umountModules
        ).let {
            return setAppProfile(it)
        }
    }

    fun isDefaultUmountModules(): Boolean {
        getAppProfile(NON_ROOT_DEFAULT_PROFILE_KEY, NOBODY_UID).let {
            return it.umountModules
        }
    }

    /** 没有内核 UAPI 可对齐：报 0，版本不匹配的提示就不会出现。 */
    val kernelUAPIVersion: Int
        get() = 0

    val managerUAPIVersion: Int
        get() = 0

    /**
     * Root through some other means, no SukiSU kernel. Kernel calls just return defaults here,
     * so the pages can be opened and browsed.
     */
    fun isPreviewMode(): Boolean = !isManager && rootAvailable()

    /**
     * 有 root 就算可用。原来的判断靠内核版本与 UAPI 对齐，这一版没有内核
     * 模块，能拿到 root 就是全部的前提。
     */
    fun isFullFeatured(): Boolean = rootAvailable()

    @Keep
    @Immutable
    @Parcelize
    @Serializable
    data class Profile(
        // and there is a default profile for root and non-root
        val name: String,
        // current uid for the package, this is convivent for kernel to check
        // if the package name doesn't match uid, then it should be invalidated.
        val currentUid: Int = 0,

        // if this is true, kernel will grant root permission to this package
        val allowSu: Boolean = false,

        // these are used for root profile
        val rootUseDefault: Boolean = true,
        val rootTemplate: String? = null,
        val uid: Int = ROOT_UID,
        val gid: Int = ROOT_GID,
        val groups: List<Int> = mutableListOf(),
        val capabilities: List<Int> = mutableListOf(),
        val context: String = KERNEL_SU_DOMAIN,
        val namespace: Int = Namespace.INHERITED.ordinal,

        val nonRootUseDefault: Boolean = true,
        val umountModules: Boolean = true,
        var rules: String = "", // this field is save in ksud!!

        val flags: Long = FLAG_KSU_NO_NEW_PRIVS,
    ) : Parcelable {
        @Keep
        enum class RootProfileFlag(val display: String, val desc: Int) {
            NO_NEW_PRIVS(
                "NO_NEW_PRIVS",
                R.string.profile_flags_desc_no_new_privs
            )
        }

        enum class Namespace {
            INHERITED,
            GLOBAL,
            INDIVIDUAL,
        }

        constructor() : this("")
    }

    const val FLAG_KSU_NO_NEW_PRIVS = 1L
}

fun List<RootProfileFlag>.toRawFlags(): Long =
    fold(0L) { acc, flag -> acc.or(1L.shl(flag.ordinal)) }

fun List<RootProfileFlag>.toOrdinalList(): List<Int> =
    map { it.ordinal }

fun Long.toRootProfileFlags(): List<RootProfileFlag> =
    RootProfileFlag.entries.filter { 1L.shl(it.ordinal).and(this) != 0L }.toList()
