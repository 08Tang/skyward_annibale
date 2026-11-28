package com.tkexig.skyward.ui.screen.settings

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.add
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.BugReport
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.DeveloperMode
import androidx.compose.material.icons.rounded.DisplaySettings
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Language
import androidx.compose.material.icons.rounded.Palette
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.tkexig.skyward.R
import com.tkexig.skyward.ui.UiMode
import com.tkexig.skyward.ui.component.KsuIsValid
import com.tkexig.skyward.ui.component.Storefront
import com.tkexig.skyward.ui.component.dialog.rememberLoadingDialog
import com.tkexig.skyward.ui.component.miuix.AppCard
import com.tkexig.skyward.ui.component.miuix.AppScaffold
import com.tkexig.skyward.ui.component.miuix.SendLogDialog
import com.tkexig.skyward.ui.theme.LocalBackgroundImagePath
import com.tkexig.skyward.ui.theme.LocalEnableBlur
import com.tkexig.skyward.ui.theme.backgroundAwareCardColors
import com.tkexig.skyward.ui.util.BlurredBar
import com.tkexig.skyward.ui.util.CustomTextKey
import com.tkexig.skyward.ui.util.LocaleHelper
import com.tkexig.skyward.ui.util.customText
import com.tkexig.skyward.ui.util.rememberBlurBackdrop
import com.tkexig.skyward.ui.viewmodel.MainPageRequest
import kotlinx.coroutines.delay
import top.yukonga.miuix.kmp.basic.Icon
import top.yukonga.miuix.kmp.basic.MiuixScrollBehavior
import top.yukonga.miuix.kmp.basic.TopAppBar
import top.yukonga.miuix.kmp.blur.layerBackdrop
import top.yukonga.miuix.kmp.preference.ArrowPreference
import top.yukonga.miuix.kmp.preference.OverlayDropdownPreference
import top.yukonga.miuix.kmp.theme.MiuixTheme.colorScheme
import top.yukonga.miuix.kmp.utils.overScrollVertical
import top.yukonga.miuix.kmp.utils.scrollEndHaptic
import kotlin.time.Duration.Companion.milliseconds

/**
 * @author weishu
 * @date 2023/1/1.
 */
@Composable
fun SettingPagerMiuix(
    uiState: SettingsUiState,
    actions: SettingsScreenActions,
    bottomInnerPadding: Dp,
    sectionRequest: MainPageRequest? = null,
    onSectionConsumed: (Long) -> Unit = {},
) {
    val scrollBehavior = MiuixScrollBehavior()
    val anchorRequesters = remember { mutableMapOf<String, BringIntoViewRequester>() }
    fun requester(anchor: String): BringIntoViewRequester =
        anchorRequesters.getOrPut(anchor) { BringIntoViewRequester() }

    LaunchedEffect(sectionRequest?.token) {
        val request = sectionRequest ?: return@LaunchedEffect
        delay(120.milliseconds)
        val anchor = SettingsSection.normalize(request.section)
        val target = anchorRequesters[SettingsSection.cardFor(anchor)]
            ?: anchorRequesters[SettingsSection.GENERAL]
        target?.bringIntoView()
        onSectionConsumed(request.token)
    }
    
    val enableBlur = LocalEnableBlur.current
    val backdrop = rememberBlurBackdrop(enableBlur)
    val hasCustomBackground = LocalBackgroundImagePath.current.isNotEmpty()
    val loadingDialog = rememberLoadingDialog()
    val showSendLogDialog = rememberSaveable { mutableStateOf(false) }

    AppScaffold(
        containerColor = if (hasCustomBackground) Color.Transparent else colorScheme.surface,
        topBar = {
            BlurredBar(backdrop = backdrop, scrollBehavior = scrollBehavior) {
                TopAppBar(
                    color = Color.Transparent,
                    title = customText(CustomTextKey.PAGE_SETTINGS),
                    scrollBehavior = scrollBehavior
                )
            }
        },
        popupHost = { },
        contentWindowInsets = WindowInsets.systemBars.add(WindowInsets.displayCutout).only(WindowInsetsSides.Horizontal),
    ) { innerPadding ->
        Box(modifier = if (backdrop != null) Modifier.layerBackdrop(backdrop) else Modifier) {
            LazyColumn(
                modifier = Modifier
                    .fillMaxHeight()
                    .scrollEndHaptic()
                    .overScrollVertical()
                    .nestedScroll(scrollBehavior.nestedScrollConnection)
                    .padding(horizontal = 12.dp),
                contentPadding = innerPadding,
                overscrollEffect = null,
            ) {
                item {
                    AppCard(
                        modifier = Modifier
                            .bringIntoViewRequester(requester(SettingsSection.GENERAL))
                            .padding(top = 12.dp)
                            .fillMaxWidth(),
                        colors = backgroundAwareCardColors(),
                    ) {
                        OverlayDropdownPreference(
                            title = customText(CustomTextKey.SETTING_UI_MODE),
                            summary = stringResource(id = R.string.settings_ui_mode_summary),
                            items = UiMode.entries.map { it.name },
                            startAction = {
                                Icon(
                                    Icons.Rounded.DisplaySettings,
                                    modifier = Modifier.padding(end = 6.dp),
                                    contentDescription = customText(CustomTextKey.SETTING_UI_MODE),
                                    tint = colorScheme.onBackground
                                )
                            },
                            selectedIndex = if (uiState.uiMode == UiMode.Material.value) 1 else 0,
                            onSelectedIndexChange = actions.onSetUiModeIndex
                        )
                        val languageSystemLabel = stringResource(id = R.string.settings_language_system)
                        val languageTags = remember { listOf(LocaleHelper.SYSTEM) + LocaleHelper.SUPPORTED_TAGS }
                        val languageNames = remember(languageSystemLabel) {
                            languageTags.map { if (it.isEmpty()) languageSystemLabel else LocaleHelper.displayName(it) }
                        }
                        OverlayDropdownPreference(
                            title = customText(CustomTextKey.SETTING_LANGUAGE),
                            summary = stringResource(id = R.string.settings_language_summary),
                            items = languageNames,
                            startAction = {
                                Icon(
                                    Icons.Rounded.Language,
                                    modifier = Modifier.padding(end = 6.dp),
                                    contentDescription = customText(CustomTextKey.SETTING_LANGUAGE),
                                    tint = colorScheme.onBackground
                                )
                            },
                            selectedIndex = languageTags.indexOf(uiState.appLanguage).coerceAtLeast(0),
                            onSelectedIndexChange = { index -> actions.onSetLanguage(languageTags[index]) }
                        )
                        val themeTitle = customText(CustomTextKey.PAGE_APPEARANCE)
                        ArrowPreference(
                            title = themeTitle,
                            summary = stringResource(id = R.string.settings_theme_summary),
                            startAction = {
                                Icon(
                                    Icons.Rounded.Palette,
                                    modifier = Modifier.padding(end = 6.dp),
                                    contentDescription = themeTitle,
                                    tint = colorScheme.onBackground
                                )
                            },
                            onClick = actions.onOpenTheme
                        )
                        val themeStoreTitle = customText(CustomTextKey.PAGE_THEME_STORE)
                        ArrowPreference(
                            title = themeStoreTitle,
                            summary = stringResource(id = R.string.theme_store_summary),
                            startAction = {
                                Icon(
                                    Icons.Rounded.Storefront,
                                    modifier = Modifier.padding(end = 6.dp),
                                    contentDescription = themeStoreTitle,
                                    tint = colorScheme.onBackground
                                )
                            },
                            onClick = actions.onOpenThemeStore
                        )

                    }

                    KsuIsValid {
                        val toolsTitle = customText(CustomTextKey.PAGE_TOOLS)
                        AppCard(
                            modifier = Modifier
                                .bringIntoViewRequester(requester(SettingsSection.TOOLS))
                                .padding(top = 12.dp)
                                .fillMaxWidth(),
                            colors = backgroundAwareCardColors(),
                        ) {
                            ArrowPreference(
                                title = toolsTitle,
                                summary = stringResource(id = R.string.settings_tools_summary),
                                startAction = {
                                    Icon(
                                        Icons.Rounded.DeveloperMode,
                                        modifier = Modifier.padding(end = 6.dp),
                                        contentDescription = toolsTitle,
                                        tint = colorScheme.onBackground
                                    )
                                },
                                onClick = {
                                    actions.onOpenTools()
                                }
                            )
                        }
                    }

                    if (uiState.isLkmMode) {
                        AppCard(
                            modifier = Modifier
                                .bringIntoViewRequester(requester(SettingsSection.UNINSTALL))
                                .padding(top = 12.dp)
                                .fillMaxWidth(),
                            colors = backgroundAwareCardColors(),
                        ) {
                            val uninstall = customText(CustomTextKey.SETTING_UNINSTALL)
                            ArrowPreference(
                                title = uninstall,
                                enabled = !uiState.isLateLoadMode,
                                startAction = {
                                    Icon(
                                        Icons.Rounded.Delete,
                                        modifier = Modifier.padding(end = 6.dp),
                                        contentDescription = uninstall,
                                        tint = colorScheme.onBackground,
                                    )
                                },
                            )
                        }
                    }

                    AppCard(
                        modifier = Modifier
                            .bringIntoViewRequester(requester(SettingsSection.DIAGNOSTICS))
                            .padding(vertical = 12.dp)
                            .fillMaxWidth(),
                        colors = backgroundAwareCardColors(),
                    ) {
                        ArrowPreference(
                            title = customText(CustomTextKey.SETTING_SEND_LOG),
                            startAction = {
                                Icon(
                                    Icons.Rounded.BugReport,
                                    modifier = Modifier.padding(end = 6.dp),
                                    contentDescription = customText(CustomTextKey.SETTING_SEND_LOG),
                                    tint = colorScheme.onBackground
                                )
                            },
                            onClick = { showSendLogDialog.value = true },
                        )
                        SendLogDialog(
                            show = showSendLogDialog.value,
                            onDismissRequest = { showSendLogDialog.value = false },
                            loadingDialog = loadingDialog
                        )
                        val about = customText(CustomTextKey.PAGE_ABOUT)
                        ArrowPreference(
                            title = about,
                            startAction = {
                                Icon(
                                    Icons.Rounded.Info,
                                    modifier = Modifier.padding(end = 6.dp),
                                    contentDescription = about,
                                    tint = colorScheme.onBackground
                                )
                            },
                            onClick = actions.onOpenAbout,
                        )
                    }
                    Spacer(Modifier.height(bottomInnerPadding))
                }
            }
        }
    }
}
