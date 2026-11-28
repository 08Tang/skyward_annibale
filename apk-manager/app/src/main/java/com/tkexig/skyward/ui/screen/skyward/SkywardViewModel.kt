package com.tkexig.skyward.ui.screen.skyward

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.tkexig.skyward.appContext
import com.tkexig.skyward.data.skyward.SkywardDevice
import com.tkexig.skyward.data.skyward.SkywardDevices
import com.tkexig.skyward.data.skyward.SkywardConfig
import com.tkexig.skyward.data.skyward.SkywardRead
import com.tkexig.skyward.data.skyward.SkywardSettings
import com.tkexig.skyward.ui.util.rootAvailable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Skyward 自己那份 settings.json 的读写。
 *
 * 配置在 /metadata/skyward/settings.json，由 Skyward 用 serde 序列化。那边
 * 的 `Settings` 整个结构体带 `#[serde(default)]`，缺字段不会读不出来；但写
 * 回去时仍然把每个字段都带上（encodeDefaults），这样文件是完整的一份，人也
 * 看得懂。
 */
data class SkywardUiState(
    val loading: Boolean = true,
    val hasRoot: Boolean = false,
    val exists: Boolean = false,
    val settings: SkywardSettings? = null,
    val error: String? = null,
    /** 刚导入但还没写回去的内容，用来先让用户看一眼再决定。 */
    val pendingImport: String? = null,
    val message: String? = null,
    /** 这台机器是哪一个；认不出来就是 null。 */
    val device: SkywardDevice? = null,
    /** 所有已知机型，用来在认不出来时说一句「只有这些」。 */
    val knownDevices: List<SkywardDevice> = emptyList(),
    /** 这台机器的代号，认不出机型时显示，方便用户报适配。 */
    val codenameOfThisDevice: String = "",
)

class SkywardViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(SkywardUiState())
    val uiState: StateFlow<SkywardUiState> = _uiState.asStateFlow()

    init {
        refresh()
    }

    fun refresh() {
        viewModelScope.launch {
            _uiState.update { it.copy(loading = true, error = null) }
            // 机型不依赖 root，先认出来，读配置失败时也能显示是哪个机型。
            // 用户选过就用他选的，没选过才按代号自动认。
            val saved = withContext(Dispatchers.IO) { SkywardDevices.savedCodename(appContext) }
            val known = withContext(Dispatchers.IO) { SkywardDevices.all(appContext) }
            val device = withContext(Dispatchers.IO) {
                if (saved.isBlank()) SkywardDevices.current(appContext)
                else known.firstOrNull { it.codename == saved }
            }
            _uiState.update {
                it.copy(
                    device = device,
                    knownDevices = known,
                    codenameOfThisDevice = SkywardDevices.deviceCodename(),
                )
            }
            val hasRoot = withContext(Dispatchers.IO) { rootAvailable() }
            if (!hasRoot) {
                _uiState.update {
                    it.copy(loading = false, hasRoot = false, error = "没有 root，读不到 Skyward 的配置。")
                }
                return@launch
            }
            val read = withContext(Dispatchers.IO) { SkywardConfig.read() }
            _uiState.update {
                when (read) {
                    is SkywardRead.Loaded -> it.copy(
                        loading = false,
                        hasRoot = true,
                        exists = true,
                        settings = read.settings,
                        error = null,
                    )

                    SkywardRead.Missing -> it.copy(
                        loading = false,
                        hasRoot = true,
                        exists = false,
                        settings = SkywardSettings(),
                        error = null,
                        message = "还没有配置文件，下面是 Skyward 的默认值",
                    )

                    // 读失败时把原因摆出来：界面上是默认值，但用户得知道那不是
                    // 他的配置。直接保存会覆盖原文件。
                    is SkywardRead.Failed -> it.copy(
                        loading = false,
                        hasRoot = true,
                        exists = true,
                        settings = SkywardSettings(),
                        error = read.reason + "。现在显示的是默认值，直接保存会覆盖原文件。",
                    )
                }
            }
        }
    }

    /** 改一个字段。界面上的每次调整都走这里，再点保存才落盘。 */
    /**
     * 手动指定机型。
     *
     * 传 null 回到自动识别的结果。只影响界面显示哪些项，不动配置文件——
     * 配置项是同一份，换机型只是换个视角。
     */
    fun selectDevice(device: SkywardDevice?) {
        // 记住这次选择：下次打开还是这台机器。传 null 是「自动」，存空串。
        SkywardDevices.saveCodename(appContext, device?.codename.orEmpty())
        _uiState.update { it.copy(device = device) }
    }

    fun edit(transform: (SkywardSettings) -> SkywardSettings) {
        _uiState.update { state ->
            val current = state.settings ?: return@update state
            state.copy(settings = transform(current), message = null)
        }
    }

    fun save() {
        val settings = _uiState.value.settings ?: return
        viewModelScope.launch {
            val result = withContext(Dispatchers.IO) { SkywardConfig.write(settings) }
            _uiState.update { state ->
                result.fold(
                    onSuccess = { state.copy(message = "已写入 Skyward 配置") },
                    onFailure = { e -> state.copy(error = e.message ?: "写入失败") },
                )
            }
        }
    }

    /** 导出成 JSON 文本，交给调用方去分享或存文件。 */
    suspend fun exportJson(): String = withContext(Dispatchers.IO) {
        val settings = _uiState.value.settings
        when {
            settings != null -> SkywardSettings.json.encodeToString(SkywardSettings.serializer(), settings)
            else -> SkywardConfig.exportJson().orEmpty()
        }
    }

    /** 先解析，成功了才让用户确认写入——免得把一份读不懂的配置盖上去。 */
    fun stageImport(text: String) {
        val parsed = runCatching {
            SkywardSettings.json.decodeFromString(SkywardSettings.serializer(), text)
        }
        _uiState.update { state ->
            parsed.fold(
                onSuccess = { state.copy(settings = it, pendingImport = text, message = "已读入，确认后写入", error = null) },
                onFailure = { e -> state.copy(error = "这份 JSON 读不懂：${e.message}") },
            )
        }
    }

    fun clearMessage() {
        _uiState.update { it.copy(message = null, error = null) }
    }
}
