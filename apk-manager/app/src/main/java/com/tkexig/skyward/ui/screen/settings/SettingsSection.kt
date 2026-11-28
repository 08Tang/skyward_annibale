package com.tkexig.skyward.ui.screen.settings

/**
 * 设置页里各张卡片的定位锚点。
 *
 * 页面上每张卡片注册一个 requester，滚动时按锚点找它。锚点也允许指到卡片
 * 里面某一行，那种情况由 [cardFor] 折回所属的卡片。
 */
object SettingsSection {

    const val GENERAL = "general"
    const val UNINSTALL = "uninstall"
    const val DIAGNOSTICS = "diagnostics"

    const val UI_MODE = "ui_mode"
    const val LANGUAGE = "language"
    const val THEME = "theme"

    val ALL: Set<String> = setOf(
        GENERAL, UNINSTALL, DIAGNOSTICS,
        UI_MODE, LANGUAGE, THEME,
    )

    fun cardFor(anchor: String): String = when (anchor.trim().lowercase().replace('-', '_')) {
        UI_MODE, LANGUAGE, THEME -> GENERAL
        UNINSTALL -> UNINSTALL
        DIAGNOSTICS -> DIAGNOSTICS
        else -> GENERAL
    }

    fun normalize(anchor: String): String {
        val id = anchor.trim().lowercase().replace('-', '_')
        return if (id in ALL) id else GENERAL
    }
}
