package com.tkexig.skyward.ui.screen.settings

import com.tkexig.skyward.ui.util.customText
import com.tkexig.skyward.ui.util.CustomTextKey
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.BugReport
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.DisplaySettings
import androidx.compose.material.icons.filled.Fence
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Palette
import com.tkexig.skyward.ui.component.Storefront
import androidx.compose.material.icons.rounded.Language
import androidx.compose.material3.Icon
import androidx.compose.material3.LargeFlexibleTopAppBar
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.TopAppBarScrollBehavior
import androidx.compose.material3.rememberTopAppBarState
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.tkexig.skyward.R
import com.tkexig.skyward.ui.UiMode
import com.tkexig.skyward.ui.component.RootReadyCheck
import com.tkexig.skyward.ui.component.material.ExpressiveScaffold
import com.tkexig.skyward.ui.component.material.IconTile
import com.tkexig.skyward.ui.component.material.SegmentedColumn
import com.tkexig.skyward.ui.component.material.SegmentedDropdownItem
import com.tkexig.skyward.ui.component.material.SegmentedListItem
import com.tkexig.skyward.ui.component.material.SendLogBottomSheet
import com.tkexig.skyward.ui.component.material.SnackBarHost
import com.tkexig.skyward.ui.component.material.expressiveTopAppBarColors
import com.tkexig.skyward.ui.theme.LocalBackgroundImagePath
import com.tkexig.skyward.ui.util.LocaleHelper
import com.tkexig.skyward.ui.viewmodel.MainPageRequest
import kotlinx.coroutines.delay
import kotlin.time.Duration.Companion.milliseconds

/**
 * @author weishu
 * @date 2023/1/1.
 */
@Composable
fun SettingPagerMaterial(
    uiState: SettingsUiState,
    actions: SettingsScreenActions,
    bottomInnerPadding: Dp,
    sectionRequest: MainPageRequest? = null,
    onSectionConsumed: (Long) -> Unit = {},
) {
    val scrollBehavior = TopAppBarDefaults.exitUntilCollapsedScrollBehavior(rememberTopAppBarState())
    val anchorRequesters = remember { mutableMapOf<String, BringIntoViewRequester>() }

    LaunchedEffect(sectionRequest?.token) {
        val request = sectionRequest ?: return@LaunchedEffect
        delay(120.milliseconds)
        val anchor = SettingsSection.normalize(request.section)
        val requester = anchorRequesters[SettingsSection.cardFor(anchor)]
            ?: anchorRequesters[SettingsSection.GENERAL]
        requester?.bringIntoView()
        onSectionConsumed(request.token)
    }
    val snackBarHost = remember { SnackbarHostState() }
    var showBottomSheet by remember { mutableStateOf(false) }

    val hasCustomBackground = LocalBackgroundImagePath.current.isNotEmpty()
    ExpressiveScaffold(
        containerColor = if (hasCustomBackground) Color.Transparent else MaterialTheme.colorScheme.surfaceContainer,
        topBar = {
            TopBar(scrollBehavior = scrollBehavior)
        },
        snackbarHost = { SnackBarHost(hostState = snackBarHost, modifier = Modifier.padding(bottom = bottomInnerPadding)) },
        contentWindowInsets = WindowInsets.safeDrawing.only(WindowInsetsSides.Top + WindowInsetsSides.Horizontal)
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .padding(paddingValues)
                .nestedScroll(scrollBehavior.nestedScrollConnection)
                .verticalScroll(rememberScrollState())
        ) {
            SettingsAnchor(SettingsSection.GENERAL, anchorRequesters) {
            SegmentedColumn(
                modifier = Modifier.padding(start = 16.dp, end = 16.dp, bottom = 13.dp),
                content = buildList {
                    add {
                        SegmentedDropdownItem(
                            icon = Icons.Filled.DisplaySettings,
                            title = customText(CustomTextKey.SETTING_UI_MODE),
                            summary = stringResource(id = R.string.settings_ui_mode_summary),
                            items = UiMode.entries.map { it.name },
                            selectedIndex = if (uiState.uiMode == UiMode.Material.value) 1 else 0,
                            onItemSelected = actions.onSetUiModeIndex
                        )
                    }
                    add {
                        val languageSystemLabel = stringResource(id = R.string.settings_language_system)
                        val languageTags = remember { listOf(LocaleHelper.SYSTEM) + LocaleHelper.SUPPORTED_TAGS }
                        val languageNames = remember(languageSystemLabel) {
                            languageTags.map { if (it.isEmpty()) languageSystemLabel else LocaleHelper.displayName(it) }
                        }
                        SegmentedDropdownItem(
                            icon = Icons.Rounded.Language,
                            title = customText(CustomTextKey.SETTING_LANGUAGE),
                            summary = stringResource(id = R.string.settings_language_summary),
                            items = languageNames,
                            selectedIndex = languageTags.indexOf(uiState.appLanguage).coerceAtLeast(0),
                            onItemSelected = { index -> actions.onSetLanguage(languageTags[index]) }
                        )
                    }
                    add {
                        val themeTitle = customText(CustomTextKey.PAGE_APPEARANCE)
                        SegmentedListItem(
                            onClick = actions.onOpenTheme,
                            headlineContent = { Text(themeTitle) },
                            supportingContent = { Text(stringResource(id = R.string.settings_theme_summary)) },
                            leadingContent = { IconTile(Icons.Filled.Palette, themeTitle) },
                            trailingContent = {
                                Icon(
                                    Icons.AutoMirrored.Filled.KeyboardArrowRight,
                                    null
                                )
                            }
                        )
                    }
                    add {
                        val themeStoreTitle = customText(CustomTextKey.PAGE_THEME_STORE)
                        SegmentedListItem(
                            onClick = actions.onOpenThemeStore,
                            headlineContent = { Text(themeStoreTitle) },
                            supportingContent = { Text(stringResource(id = R.string.theme_store_summary)) },
                            leadingContent = { IconTile(Icons.Filled.Storefront, themeStoreTitle) },
                            trailingContent = {
                                Icon(
                                    Icons.AutoMirrored.Filled.KeyboardArrowRight,
                                    null
                                )
                            }
                        )
                    }

                }
            )
            }


            if (uiState.isLkmMode) {
                SettingsAnchor(SettingsSection.UNINSTALL, anchorRequesters) {
                SegmentedColumn(
                    modifier = Modifier.padding(start = 16.dp, end = 16.dp, bottom = 13.dp),
                    content = listOf(
                        {
                            val uninstall = customText(CustomTextKey.SETTING_UNINSTALL)
                            SegmentedListItem(
                                enabled = !uiState.isLateLoadMode,
                                headlineContent = { Text(uninstall) },
                                leadingContent = { IconTile(Icons.Filled.Delete, uninstall) }
                            )
                        }
                    )
                )
                }
            }

            SettingsAnchor(SettingsSection.DIAGNOSTICS, anchorRequesters) {
            SegmentedColumn(
                modifier = Modifier.padding(start = 16.dp, end = 16.dp, bottom = 13.dp),
                content = listOf(
                    {
                        SegmentedListItem(
                            onClick = { showBottomSheet = true },
                            headlineContent = { Text(customText(CustomTextKey.SETTING_SEND_LOG)) },
                            leadingContent = {
                                IconTile(
                                    Icons.Filled.BugReport,
                                    customText(CustomTextKey.SETTING_SEND_LOG)
                                )
                            },
                        )
                    },
                    {
                        val aboutTitle = customText(CustomTextKey.PAGE_ABOUT)
                        SegmentedListItem(
                            onClick = actions.onOpenAbout,
                            headlineContent = { Text(aboutTitle) },
                            leadingContent = {
                                IconTile(
                                    Icons.Filled.Info,
                                    aboutTitle
                                )
                            },
                        )
                    }
                )
            )
            }

            Spacer(modifier = Modifier.height(8.dp))

            if (showBottomSheet) {
                SendLogBottomSheet(
                    onDismiss = { showBottomSheet = false },
                    snackbarHostState = snackBarHost,
                )
            }
            Spacer(modifier = Modifier.height(bottomInnerPadding))
        }
    }
}

@Composable
private fun SettingsAnchor(
    anchor: String,
    registry: MutableMap<String, BringIntoViewRequester>,
    content: @Composable () -> Unit,
) {
    val requester = remember { BringIntoViewRequester() }
    SideEffect { registry[anchor] = requester }
    Box(modifier = Modifier.fillMaxWidth().bringIntoViewRequester(requester)) {
        content()
    }
}

@Composable
private fun TopBar(
    scrollBehavior: TopAppBarScrollBehavior? = null
) {
    val hasCustomBackground = LocalBackgroundImagePath.current.isNotEmpty()
    LargeFlexibleTopAppBar(
        title = { Text(customText(CustomTextKey.PAGE_SETTINGS)) },
        colors = expressiveTopAppBarColors(
            containerColor = if (hasCustomBackground) Color.Transparent else MaterialTheme.colorScheme.surfaceContainer,
            scrolledContainerColor = if (hasCustomBackground) Color.Transparent else MaterialTheme.colorScheme.surfaceContainer,
        ),
        windowInsets = WindowInsets.safeDrawing.only(WindowInsetsSides.Top + WindowInsetsSides.Horizontal),
        scrollBehavior = scrollBehavior
    )
}
