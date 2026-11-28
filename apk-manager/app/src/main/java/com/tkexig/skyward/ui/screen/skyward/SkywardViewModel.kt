package com.tkexig.skyward.ui.screen.skyward

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.tkexig.skyward.data.skyward.SkywardConfig
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
 * 有一半字段没有 #[serde(default)]，所以写回去时必须把每个字段都带上——
 * 缺一个 Skyward 就解析失败，用户的设置会被它当成损坏丢掉。Kotlin 这边
 * 靠 encodeDefaults 保证这一点。
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
            val hasRoot = withContext(Dispatchers.IO) { rootAvailable() }
            if (!hasRoot) {
                _uiState.update {
                    it.copy(loading = false, hasRoot = false, error = "没有 root，读不到 Skyward 的配置。")
                }
                return@launch
            }
            val result = runCatching { withContext(Dispatchers.IO) { SkywardConfig.read() } }
            result.fold(
                onSuccess = { parsed ->
                    _uiState.update {
                        it.copy(
                            loading = false,
                            hasRoot = true,
                            exists = parsed != null,
                            settings = parsed ?: SkywardSettings(),
                            error = null,
                        )
                    }
                },
                onFailure = { e ->
                    _uiState.update {
                        it.copy(
                            loading = false,
                            hasRoot = true,
                            exists = false,
                            error = e.message ?: "读配置失败",
                        )
                    }
                },
            )
        }
    }

    /** 改一个字段。界面上的每次调整都走这里，再点保存才落盘。 */
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
