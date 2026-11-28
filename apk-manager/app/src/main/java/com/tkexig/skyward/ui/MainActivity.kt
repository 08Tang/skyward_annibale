package com.tkexig.skyward.ui

import android.annotation.SuppressLint
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.annotation.RequiresApi
import androidx.compose.animation.Crossfade
import androidx.compose.foundation.background
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.draw.blur
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import coil.compose.AsyncImage
import coil.request.ImageRequest
import androidx.compose.ui.platform.LocalContext
import com.tkexig.skyward.ui.component.splash.SplashScreen
import com.tkexig.skyward.ui.theme.LocalAppBackgroundBackdrop
import com.tkexig.skyward.ui.theme.LocalAppSettings
import com.tkexig.skyward.ui.theme.MaterialExpressivePalette
import com.tkexig.skyward.ui.theme.LocalBackgroundImagePath
import com.tkexig.skyward.ui.theme.LocalCardBackgroundAlpha
import com.tkexig.skyward.ui.theme.appearanceImageEffects
import java.io.File
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerDefaults.flingBehavior
import androidx.compose.foundation.pager.PagerDefaults.pageNestedScrollConnection
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.UriHandler
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigationevent.NavigationEventInfo
import androidx.navigationevent.compose.NavigationBackHandler
import androidx.navigationevent.compose.rememberNavigationEventState
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import com.tkexig.skyward.Natives
import com.tkexig.skyward.ui.component.bottombar.BottomBar
import com.tkexig.skyward.ui.component.bottombar.MainPagerState
import com.tkexig.skyward.ui.component.bottombar.NavigationBadgeState
import com.tkexig.skyward.ui.component.bottombar.SideRail
import com.tkexig.skyward.ui.component.bottombar.rememberMainPagerState
import com.tkexig.skyward.ui.component.bottombar.useNavigationRail
import com.tkexig.skyward.ui.navigation3.IntentDispatcher
import com.tkexig.skyward.ui.navigation3.LocalNavigator
import com.tkexig.skyward.ui.navigation3.Navigator
import com.tkexig.skyward.ui.navigation3.Route
import com.tkexig.skyward.ui.navigation3.rememberNavigator
import com.tkexig.skyward.ui.screen.about.AboutScreen
import com.tkexig.skyward.ui.screen.appicon.AppIconScreen
import com.tkexig.skyward.ui.screen.customtext.CustomTextScreen
import com.tkexig.skyward.ui.screen.account.AccountScreen
import com.tkexig.skyward.ui.screen.themestore.ThemeStoreScreen
import com.tkexig.skyward.ui.screen.themestore.ThemeStoreSourcesScreen
import com.tkexig.skyward.ui.util.LocalCustomTexts
import com.tkexig.skyward.ui.screen.backgroundcrop.BackgroundCropScreen
import com.tkexig.skyward.ui.screen.colorpalette.AdaptiveThemeBuilderScreen
import com.tkexig.skyward.ui.screen.colorpalette.ColorPaletteScreen
import com.tkexig.skyward.ui.screen.appearanceoverlay.AppearanceOverlayScreen
import com.tkexig.skyward.ui.screen.home.HomePager
import com.tkexig.skyward.ui.screen.settings.SettingPager
import com.tkexig.skyward.ui.screen.install.InstallRouteScreen
import com.tkexig.skyward.ui.screen.skyward.SkywardPager
import com.tkexig.skyward.ui.screen.settings.tools.ToolsScreen
import com.tkexig.skyward.ui.screen.sulog.SulogScreen
import com.tkexig.skyward.ui.theme.KernelSUTheme
import com.tkexig.skyward.ui.theme.LocalColorMode
import com.tkexig.skyward.ui.theme.LocalEnableBlur
import com.tkexig.skyward.ui.theme.LocalEnableFloatingBottomBar
import com.tkexig.skyward.ui.theme.LocalEnableFloatingBottomBarBlur
import com.tkexig.skyward.ui.theme.LocalEnableNavigationBadge
import com.tkexig.skyward.ui.theme.LocalEnableProgressiveBlur
import com.tkexig.skyward.ui.theme.LocalModuleDescriptionMaxLines
import com.tkexig.skyward.ui.util.BackgroundNavTransition
import com.tkexig.skyward.ui.util.ImageStorage
import com.tkexig.skyward.ui.util.LocaleHelper
import com.tkexig.skyward.ui.util.ThemeTrialManager
import com.tkexig.skyward.ui.util.rememberBlurBackdrop
import com.tkexig.skyward.ui.util.rememberContentReady
import com.tkexig.skyward.ui.viewmodel.MainActivityViewModel
import com.tkexig.skyward.ui.viewmodel.MainPagerConfig
import com.tkexig.skyward.ui.viewmodel.SettingsViewModel
import top.yukonga.miuix.kmp.basic.Scaffold
import top.yukonga.miuix.kmp.blur.layerBackdrop
import top.yukonga.miuix.kmp.blur.rememberLayerBackdrop
import top.yukonga.miuix.kmp.nav.core.NavDisplay
import top.yukonga.miuix.kmp.nav.core.NavDisplayEffects
import top.yukonga.miuix.kmp.nav.core.rememberNavSystemCornerRadius
import top.yukonga.miuix.kmp.nav.transition.NavSwipeDirection
import top.yukonga.miuix.kmp.nav.transition.NavTransitions
import top.yukonga.miuix.kmp.shader.isRuntimeShaderSupported
import top.yukonga.miuix.kmp.theme.MiuixTheme
import top.yukonga.miuix.kmp.utils.PagerGestureNestedScrollConnection
import top.yukonga.miuix.kmp.utils.PagerInterceptionMode
import top.yukonga.miuix.kmp.utils.PagerNavigationSpringSpec
import top.yukonga.miuix.kmp.utils.pagerGestureOverride
import androidx.core.graphics.drawable.toDrawable
import com.tkexig.skyward.R


class MainActivity : ComponentActivity() {
    private val intentChannel = Channel<Intent>(capacity = Channel.BUFFERED)

    override fun attachBaseContext(newBase: android.content.Context) {
        super.attachBaseContext(LocaleHelper.wrap(newBase))
    }

    @RequiresApi(Build.VERSION_CODES.Q)
    @SuppressLint("UnusedMaterial3ScaffoldPaddingParameter")
    override fun onCreate(savedInstanceState: Bundle?) {
        installSplashScreen()
        super.onCreate(savedInstanceState)
        if (ThemeTrialManager.isActive(this)) {
            ViewModelProvider(this)[SettingsViewModel::class.java]
                .restoreThemeTrial(this, taskId)
        } else {
            ThemeTrialManager.cleanupOrphan(this)
        }

        val isManager = Natives.isManager

        if (savedInstanceState == null) intent?.let { intentChannel.trySend(it) }

        setContent {
            val viewModel = viewModel<MainActivityViewModel>()
            val uiState by viewModel.uiState.collectAsStateWithLifecycle()
            val selectedMainPage by viewModel.selectedMainPage.collectAsStateWithLifecycle()
            val appSettings = uiState.appSettings
            val uiMode = uiState.uiMode
            val darkMode = appSettings.colorMode.isDark || (appSettings.colorMode.isSystem && isSystemInDarkTheme())

            // Pick up launcher wallpaper changes while the "use system wallpaper" option is on.
            LifecycleEventEffect(Lifecycle.Event.ON_RESUME) {
                viewModel.refreshSystemWallpaper()
                // Also covers a mode change that happened while the app was in the background.
                viewModel.syncAdaptiveTheme(darkMode)
            }

            // Adaptive theme packages carry a light and a dark half; swap them with the effective
            // light/dark mode (system-following or a forced app mode).
            LaunchedEffect(darkMode) {
                viewModel.syncAdaptiveTheme(darkMode)
            }

            DisposableEffect(darkMode) {
                enableEdgeToEdge(
                    statusBarStyle = SystemBarStyle.auto(
                        android.graphics.Color.TRANSPARENT,
                        android.graphics.Color.TRANSPARENT
                    ) { darkMode },
                    navigationBarStyle = SystemBarStyle.auto(
                        android.graphics.Color.TRANSPARENT,
                        android.graphics.Color.TRANSPARENT
                    ) { darkMode },
                )
                window.isNavigationBarContrastEnforced = false
                onDispose { }
            }

            LaunchedEffect(darkMode) {
                window.setBackgroundDrawable(
                    getColor(if (darkMode) android.R.color.background_dark else android.R.color.background_light).toDrawable()
                )
            }

            val navigator = rememberNavigator(Route.Main)
            val context = LocalContext.current
            val uriHandler = LocalUriHandler.current
            val systemDensity = LocalDensity.current
            val density = remember(systemDensity, uiState.pageScale) {
                Density(systemDensity.density * uiState.pageScale, systemDensity.fontScale)
            }
            // Bars sample the page backdrop, which is transparent wherever the page paints nothing.
            // Blurring that reads as black, so over a custom background they also need the wallpaper:
            // capture it here, where only the background image is drawn, and combine it underneath.
            val appBackgroundBackdrop = rememberLayerBackdrop()
            // The progressive blur covers both shells and works regardless of the plain bar blur.
            val progressiveBlurActive = uiState.enableProgressiveBlur && isRuntimeShaderSupported()
            val captureAppBackground = uiState.customBackgroundUri.isNotEmpty() &&
                ((uiMode == UiMode.Miuix && uiState.enableBlur) || progressiveBlurActive)

            val themedAppSettings = remember(appSettings, uiState.keyColorOverride) {
                uiState.keyColorOverride?.let { appSettings.copy(keyColor = it) } ?: appSettings
            }
            CompositionLocalProvider(
                LocalUriHandler provides object : UriHandler {
                    override fun openUri(uri: String) {
                        try {
                            uriHandler.openUri(uri)
                        } catch (_: IllegalArgumentException) {
                            Toast.makeText(context, R.string.external_link_open_failed, Toast.LENGTH_SHORT).show()
                        }
                    }
                },
                LocalNavigator provides navigator,
                LocalDensity provides density,
                LocalColorMode provides appSettings.colorMode.value,
                LocalAppSettings provides themedAppSettings,
                LocalEnableBlur provides uiState.enableBlur,
                LocalEnableProgressiveBlur provides uiState.enableProgressiveBlur,
                LocalEnableFloatingBottomBar provides uiState.enableFloatingBottomBar,
                LocalEnableFloatingBottomBarBlur provides uiState.enableFloatingBottomBarBlur,
                LocalEnableNavigationBadge provides uiState.enableNavigationBadge,
                LocalModuleDescriptionMaxLines provides uiState.moduleDescriptionMaxLines,
                LocalBackgroundImagePath provides uiState.customBackgroundUri,
                LocalAppBackgroundBackdrop provides appBackgroundBackdrop.takeIf { captureAppBackground },
                LocalCardBackgroundAlpha provides (uiState.cardBackgroundAlphaPercent / 100f),
                LocalUiMode provides uiMode,
                LocalCustomTexts provides uiState.customTexts,
            ) {
                KernelSUTheme(appSettings = themedAppSettings, uiMode = uiMode) {
                    IntentDispatcher(intentChannel = intentChannel)
                    val swipeDismiss = if (uiState.enableSwipeDismiss) {
                        if (LocalLayoutDirection.current == androidx.compose.ui.unit.LayoutDirection.Rtl) {
                            NavSwipeDirection.RightToLeft
                        } else {
                            NavSwipeDirection.LeftToRight
                        }
                    } else {
                        NavSwipeDirection.None
                    }
                    val mainScreenEntry = @Composable {
                        MainScreen(
                            initialPage = selectedMainPage,
                            pagerInterceptionMode = uiState.pagerInterceptionMode,
                            onPageChanged = viewModel::setSelectedMainPage,
                        )
                    }

                    val navDisplay = @Composable {
                        val backgroundActive = uiState.customBackgroundUri.isNotEmpty()
                        NavDisplay(
                            backStack = navigator.backStack,
                            transition = if (backgroundActive) {
                                BackgroundNavTransition
                            } else {
                                NavTransitions.MiuixDefault
                            },
                            effects = NavDisplayEffects(
                                cornerClipRadius = rememberNavSystemCornerRadius(),
                                // The dim scrim sits under the top destination even when idle; an
                                // opaque page hid it, but a transparent one would stay darkened.
                                dimAmount = if (backgroundActive) 0f else 0.5f,
                            ),
                            onBack = { navigator.pop() }) {
                            entry<Route.Main>(swipeDismiss = swipeDismiss) { mainScreenEntry() }
                            entry<Route.About>(swipeDismiss = swipeDismiss) { AboutScreen() }
                            entry<Route.Sulog>(swipeDismiss = swipeDismiss) { SulogScreen() }
                            entry<Route.ColorPalette>(swipeDismiss = swipeDismiss) { key -> ColorPaletteScreen(key.section, key.importUri) }
                            entry<Route.AppearanceOverlay>(swipeDismiss = swipeDismiss) {
                                AppearanceOverlayScreen(onBack = { navigator.pop() })
                            }
                            entry<Route.CustomTexts>(swipeDismiss = swipeDismiss) { CustomTextScreen() }
                            entry<Route.ThemeStore>(swipeDismiss = swipeDismiss) { key ->
                                MaterialExpressivePalette {
                                    ThemeStoreScreen(initialThemeId = key.themeId, initialSourceBaseUrl = key.sourceBaseUrl)
                                }
                            }
                            entry<Route.ThemeStoreAdaptiveUpload>(swipeDismiss = swipeDismiss) {
                                MaterialExpressivePalette {
                                    ThemeStoreScreen(startAdaptiveUpload = true)
                                }
                            }
                            entry<Route.ThemeStoreCurrentUpload>(swipeDismiss = swipeDismiss) {
                                MaterialExpressivePalette {
                                    ThemeStoreScreen(startCurrentUpload = true)
                                }
                            }
                            entry<Route.ThemeStoreSources>(swipeDismiss = swipeDismiss) {
                                MaterialExpressivePalette { ThemeStoreSourcesScreen() }
                            }
                            entry<Route.ThemeAccount>(swipeDismiss = swipeDismiss) {
                                MaterialExpressivePalette { AccountScreen() }
                            }
                            entry<Route.AdaptiveThemeBuilder>(swipeDismiss = swipeDismiss) {
                                MaterialExpressivePalette { AdaptiveThemeBuilderScreen() }
                            }
                            entry<Route.AppIcon>(swipeDismiss = swipeDismiss) { AppIconScreen() }
                            entry<Route.BackgroundCrop>(swipeDismiss = NavSwipeDirection.None) { key ->
                                BackgroundCropScreen(key.sourcePath)
                            }
                            entry<Route.Home>(swipeDismiss = swipeDismiss) { mainScreenEntry() }
                            entry<Route.Settings>(swipeDismiss = swipeDismiss) { mainScreenEntry() }
                                entry<Route.Tool>(swipeDismiss = swipeDismiss)  { ToolsScreen() }
                                entry<Route.FlashRecovery>(swipeDismiss = swipeDismiss) {
                                    InstallRouteScreen(onBack = { navigator.pop() })
                                }
                        }
                    }

                    var appContentReady by remember { mutableStateOf(false) }
                    var splashDismissed by remember { mutableStateOf(false) }

                    SideEffect {
                        if (!appContentReady) appContentReady = true
                    }

                    Box(
                        modifier = Modifier.fillMaxSize()
                    ) {
                        val hasCustomBackground = uiState.customBackgroundUri.isNotEmpty()
                        val backgroundBlurRadius = (
                            ImageStorage.BACKGROUND_BLUR_MAX_RADIUS_DP * uiState.customBackgroundBlurPercent / 100f
                        ).dp
                        val backgroundEffects = themedAppSettings.appearance?.background
                        Crossfade(
                            modifier = if (captureAppBackground) {
                                Modifier.layerBackdrop(appBackgroundBackdrop)
                            } else {
                                Modifier
                            },
                            targetState = uiState.customBackgroundUri to uiState.customBackgroundUpdatedAt,
                            label = "global_background",
                        ) { (path, _) ->
                            if (path.isNotEmpty()) {
                                val context = LocalContext.current
                                val stamp = File(path).lastModified()
                                AsyncImage(
                                    model = ImageRequest.Builder(context)
                                        .data(File(path))
                                        .crossfade(true)
                                        .memoryCacheKey("$path@$stamp")
                                        .diskCacheKey("$path@$stamp")
                                        .build(),
                                    contentDescription = null,
                                    contentScale = ContentScale.Crop,
                                    modifier = Modifier
                                        .fillMaxSize()
                                        .then(
                                            if (backgroundBlurRadius > 0.dp) {
                                                Modifier.blur(backgroundBlurRadius)
                                            } else {
                                                Modifier
                                            }
                                        )
                                        // Sharpen after a blur would just fight the blur, so only
                                        // apply it while the background is unblurred.
                                        .appearanceImageEffects(
                                            backgroundEffects,
                                            allowSharpen = backgroundBlurRadius <= 0.dp,
                                        )
                                )
                                val scrimPercent = uiState.customBackgroundScrimPercent
                                if (scrimPercent != 0) {
                                    val scrimColor = if (scrimPercent < 0) {
                                        Color.Black.copy(alpha = -scrimPercent / 100f)
                                    } else {
                                        Color.White.copy(alpha = scrimPercent / 100f)
                                    }
                                    Box(
                                        modifier = Modifier
                                            .fillMaxSize()
                                            .background(scrimColor)
                                    )
                                }
                            }
                        }

                        when (uiMode) {
                            UiMode.Material -> androidx.compose.material3.Scaffold(
                                containerColor = if (hasCustomBackground) Color.Transparent else MaterialTheme.colorScheme.surfaceContainer
                            ) { navDisplay() }

                            UiMode.Miuix -> Scaffold(
                                containerColor = if (hasCustomBackground) Color.Transparent else MiuixTheme.colorScheme.surface
                            ) { navDisplay() }
                        }

                        if (!splashDismissed) {
                            SplashScreen(
                                isReady = appContentReady,
                                onDismissed = { splashDismissed = true }
                            )
                        }
                    }
                }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        intentChannel.trySend(intent)
    }
}

val LocalMainPagerState = staticCompositionLocalOf<MainPagerState> { error("LocalMainPagerState not provided") }

@RequiresApi(Build.VERSION_CODES.Q)
@SuppressLint("UnusedMaterial3ScaffoldPaddingParameter")
@Composable
fun MainScreen(
    initialPage: Int = 0,
    pagerInterceptionMode: Int = PagerInterceptionMode.CrossAxisInterceptor.ordinal,
    onPageChanged: (Int) -> Unit = {},
) {
    val navController = LocalNavigator.current
    val enableBlur = LocalEnableBlur.current
    val enableProgressiveBlur = LocalEnableProgressiveBlur.current
    val enableFloatingBottomBar = LocalEnableFloatingBottomBar.current
    val enableFloatingBottomBarBlur = LocalEnableFloatingBottomBarBlur.current
    val useNavigationRail = useNavigationRail(enableFloatingBottomBar)
    val pagerState = rememberPagerState(initialPage = initialPage, pageCount = { MainPagerConfig.PAGE_COUNT })
    val mainPagerState = rememberMainPagerState(
        pagerState = pagerState,
        animatePageChanges = !useNavigationRail,
    )
    // The AI assistant can request a tab from outside MainScreen; follow the
    // selected page when it changes without a pager gesture.
    LaunchedEffect(initialPage) {
        if (pagerState.currentPage != initialPage) {
            mainPagerState.animateToPage(initialPage)
        }
    }
    val isFullFeatured = Natives.isFullFeatured()
    val pagerMode = PagerInterceptionMode.entries.getOrElse(pagerInterceptionMode) {
        PagerInterceptionMode.Native
    }
    val interceptPagerGestures = pagerMode == PagerInterceptionMode.CrossAxisInterceptor
    var userScrollEnabled by remember(isFullFeatured) { mutableStateOf(isFullFeatured) }

    val enableNavigationBadge = LocalEnableNavigationBadge.current
    val badgeEnabled = enableNavigationBadge && isFullFeatured
    // 底栏不再有「超级用户」「模块」两页，也就没有数要标。
    val navigationBadge = NavigationBadgeState()
    val uiMode = LocalUiMode.current
    val hasCustomBackground = LocalBackgroundImagePath.current.isNotEmpty()
    val cardAlpha = LocalCardBackgroundAlpha.current
    val surfaceColor = when (uiMode) {
        UiMode.Material -> MaterialTheme.colorScheme.surfaceContainer // Matches the Material page background behind the glass bar
        UiMode.Miuix -> MiuixTheme.colorScheme.surface
    }
    // This backdrop only feeds the standard Miuix navigation bar. The floating bars use
    // `backdrop` for their liquid glass or progressive band, so the pager is only recorded into
    // a full-screen layer when one of those readers is active.
    val needsStandardMiuixBarBackdrop = uiMode == UiMode.Miuix && !enableFloatingBottomBar &&
        !useNavigationRail
    val blurBackdrop = rememberBlurBackdrop(
        enableBlur = enableBlur && needsStandardMiuixBarBackdrop,
        allowProgressive = needsStandardMiuixBarBackdrop,
    )
    // The floating pills (Material bar, Miuix floating bar) sample this pager layer for their
    // liquid glass and/or progressive blur. Rail layouts show no bottom bar at all.
    val needsBottomBarBackdrop = !useNavigationRail &&
        (uiMode == UiMode.Material || enableFloatingBottomBar) &&
        (enableFloatingBottomBarBlur || enableProgressiveBlur)

    val backdrop = rememberLayerBackdrop {
        if (!hasCustomBackground) {
            drawRect(surfaceColor)
        } else {
            drawRect(surfaceColor.copy(alpha = cardAlpha))
        }
        drawContent()
    }

    val settledPage = mainPagerState.pagerState.settledPage
    LaunchedEffect(settledPage) {
        onPageChanged(settledPage)
    }

    val currentPage = mainPagerState.pagerState.currentPage
    LaunchedEffect(currentPage) {
        mainPagerState.syncPage()
    }

    MainScreenBackHandler(mainPagerState, navController)

    CompositionLocalProvider(
        LocalMainPagerState provides mainPagerState
    ) {
        val contentReady = rememberContentReady()
        val pagerContent = @Composable { bottomInnerPadding: Dp ->
            Box(modifier = if (blurBackdrop != null) Modifier.layerBackdrop(blurBackdrop) else Modifier) {
                HorizontalPager(
                    modifier = Modifier
                        .pagerGestureOverride(
                            pagerState = mainPagerState.pagerState,
                            mode = pagerMode,
                            enabled = userScrollEnabled,
                        )
                        // Material always floats its bar, so only the glass/progressive band matters there.
                        .then(if (needsBottomBarBackdrop) Modifier.layerBackdrop(backdrop) else Modifier),
                    state = mainPagerState.pagerState,
                    beyondViewportPageCount = if (contentReady) 3 else 0,
                    overscrollEffect = null,
                    userScrollEnabled = userScrollEnabled && !interceptPagerGestures,
                    pageNestedScrollConnection = if (interceptPagerGestures) {
                        PagerGestureNestedScrollConnection
                    } else {
                        pageNestedScrollConnection(
                            state = mainPagerState.pagerState,
                            orientation = androidx.compose.foundation.gestures.Orientation.Horizontal,
                        )
                    },
                    flingBehavior = flingBehavior(
                        state = mainPagerState.pagerState,
                        snapAnimationSpec = PagerNavigationSpringSpec,
                    ),
                ) { page ->
                    val isCurrentPage = page == settledPage
                    when (page) {
                        0 -> if (contentReady || isCurrentPage) HomePager(navController, bottomInnerPadding, isCurrentPage)
                        1 -> if (contentReady || isCurrentPage) SkywardPager(bottomInnerPadding, isCurrentPage)
                        2 -> if (contentReady || isCurrentPage) SettingPager(navController, bottomInnerPadding, isCurrentPage)
                    }
                }
            }
        }

        if (useNavigationRail) {
            val startInsets = WindowInsets.systemBars.union(WindowInsets.displayCutout)
                .only(WindowInsetsSides.Start)
            val navBarBottomPadding = WindowInsets.systemBars.asPaddingValues().calculateBottomPadding()

            when (uiMode) {
                UiMode.Material -> androidx.compose.material3.Scaffold(
                    containerColor = if (hasCustomBackground) Color.Transparent else MaterialTheme.colorScheme.surfaceContainer
                ) {
                    Row {
                        SideRail(navigationBadge)
                        Box(
                            modifier = Modifier
                                .weight(1f)
                                .consumeWindowInsets(startInsets)
                        ) {
                            pagerContent(navBarBottomPadding)
                        }
                    }
                }

                UiMode.Miuix -> Scaffold(
                    containerColor = if (hasCustomBackground) Color.Transparent else MiuixTheme.colorScheme.surface
                ) { _ ->
                    Row {
                        SideRail(navigationBadge)
                        Box(
                            modifier = Modifier
                                .weight(1f)
                                .consumeWindowInsets(startInsets)
                        ) {
                            pagerContent(navBarBottomPadding)
                        }
                    }
                }
            }
        } else {
            val bottomBar = @Composable {
                Box(
                    modifier = Modifier.fillMaxWidth()
                ) {
                    BottomBar(
                        blurBackdrop = blurBackdrop,
                        backdrop = backdrop,
                        navigationBadge = navigationBadge,
                        modifier = Modifier.align(Alignment.BottomCenter),
                    )
                }
            }

            when (uiMode) {
                UiMode.Material -> androidx.compose.material3.Scaffold(
                    bottomBar = bottomBar,
                    containerColor = if (hasCustomBackground) Color.Transparent else MaterialTheme.colorScheme.surfaceContainer
                ) { innerPadding ->
                    pagerContent(innerPadding.calculateBottomPadding())
                }

                UiMode.Miuix -> Scaffold(
                    bottomBar = bottomBar,
                    containerColor = if (hasCustomBackground) Color.Transparent else MiuixTheme.colorScheme.surface
                ) { innerPadding ->
                    pagerContent(innerPadding.calculateBottomPadding())
                }
            }
        }
    }
}

@Composable
private fun MainScreenBackHandler(
    mainState: MainPagerState,
    navController: Navigator,
) {
    val isPagerBackHandlerEnabled by remember {
        derivedStateOf {
            navController.current() is Route.Main && navController.backStackSize() == 1 && mainState.selectedPage != 0
        }
    }

    val navEventState = rememberNavigationEventState(NavigationEventInfo.None)

    NavigationBackHandler(
        state = navEventState,
        isBackEnabled = isPagerBackHandlerEnabled,
        onBackCompleted = {
            mainState.animateToPage(0)
        }
    )
}
