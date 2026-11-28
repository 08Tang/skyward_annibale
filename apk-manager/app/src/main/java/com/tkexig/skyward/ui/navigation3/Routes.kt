package com.tkexig.skyward.ui.navigation3

import android.net.Uri
import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import com.tkexig.skyward.ui.util.UriSerializer
import top.yukonga.miuix.kmp.nav.core.NavKey

/**
 * Type-safe navigation keys for miuix-nav.
 * Each destination is a NavKey (data object/data class) and can be saved/restored in the back stack.
 */
@Serializable
sealed interface Route : NavKey, Parcelable {
    @Parcelize
    @Serializable
    data object Main : Route

    @Parcelize
    @Serializable
    data object Home : Route

    @Parcelize
    @Serializable
    data object Settings : Route

    @Parcelize
    @Serializable
    data object About : Route

    @Parcelize
    @Serializable
    data object Sulog : Route

    @Parcelize
    @Serializable
    data class ColorPalette(
        val section: String? = null,
        
        val importUri: String? = null,
    ) : Route

    @Parcelize
    @Serializable
    data object AppearanceOverlay : Route

    @Parcelize
    @Serializable
    data object CustomTexts : Route

    @Parcelize
    @Serializable
    data class ThemeStore(
        val themeId: String? = null,
        val sourceBaseUrl: String? = null,
    ) : Route

    @Parcelize
    @Serializable
    data object ThemeStoreAdaptiveUpload : Route

    @Parcelize
    @Serializable
    data object ThemeStoreCurrentUpload : Route

    @Parcelize
    @Serializable
    data object ThemeStoreSources : Route

    @Parcelize
    @Serializable
    data class ThemeAccount(val tab: String? = null) : Route

    @Parcelize
    @Serializable
    data object AdaptiveThemeBuilder : Route

    @Parcelize
    @Serializable
    data object AppIcon : Route

    @Parcelize
    @Serializable
    data class BackgroundCrop(val sourcePath: String) : Route

    @Parcelize
    @Serializable
    data object Tool : Route

    /** 刷入 Skyward 的 recovery 镜像。 */
    @Parcelize
    @Serializable
    data class FlashRecovery(val from: String? = null) : Route
}
