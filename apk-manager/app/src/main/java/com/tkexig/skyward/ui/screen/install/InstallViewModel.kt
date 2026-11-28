package com.tkexig.skyward.ui.screen.install

import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.tkexig.skyward.data.skyward.RecoveryFlasher
import com.tkexig.skyward.data.skyward.SkywardRelease
import com.tkexig.skyward.data.skyward.SkywardReleases
import com.tkexig.skyward.ksuApp
import com.tkexig.skyward.ui.util.rootAvailable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

/** 刷机页在做哪一步。 */
enum class InstallPhase { Idle, Downloading, Extracting, BackingUp, Flashing, Done, Failed }

data class InstallUiState(
    val loadingRelease: Boolean = true,
    val release: SkywardRelease? = null,
    val releaseError: String? = null,

    val hasRoot: Boolean = false,
    /** 这台机器有没有独立的 recovery 分区；A/B 机型通常没有。 */
    val hasRecoveryPartition: Boolean = false,
    /** 当前槽位，非 A/B 机型是空的。 */
    val slotSuffix: String = "",

    val phase: InstallPhase = InstallPhase.Idle,
    val progress: Int = 0,
    /** 给用户看的一行进展。 */
    val status: String = "",

    /** 用户自己挑的镜像；为空就用发布包里那个。 */
    val pickedImage: File? = null,
    val pickedName: String = "",
    /** 选「先备份当前 recovery」没有。 */
    val backupFirst: Boolean = true,
    /** 备份下来的文件，刷完可以告诉用户在哪。 */
    val backupFile: File? = null,

    val error: String? = null,
)

/**
 * 刷入 Skyward。
 *
 * 顺序是固定的：下载（或用户自选）→ 解出镜像 → 备份当前 recovery → 写分区。
 * 每一步失败都停下，不会拿一份没验证过的镜像去覆盖分区。
 */
class InstallViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(InstallUiState())
    val uiState: StateFlow<InstallUiState> = _uiState.asStateFlow()

    init {
        refresh()
    }

    fun refresh() {
        viewModelScope.launch {
            _uiState.update { it.copy(loadingRelease = true, releaseError = null) }
            val root = withContext(Dispatchers.IO) { rootAvailable() }
            val partition = withContext(Dispatchers.IO) {
                if (root) RecoveryFlasher.hasRecoveryPartition() else false
            }
            val slot = withContext(Dispatchers.IO) {
                if (root) RecoveryFlasher.currentSlotSuffix() else ""
            }
            _uiState.update {
                it.copy(hasRoot = root, hasRecoveryPartition = partition, slotSuffix = slot)
            }
            val release = SkywardReleases.latest()
            _uiState.update {
                it.copy(
                    loadingRelease = false,
                    release = release,
                    releaseError = if (release == null) "取不到 Skyward 的最新发布" else null,
                )
            }
        }
    }

    /** 用户从文件管理器挑了一个镜像。 */
    fun pickImage(uri: Uri) {
        viewModelScope.launch {
            val copied = withContext(Dispatchers.IO) {
                runCatching {
                    val dir = File(ksuApp.cacheDir, "skyward-install").apply { mkdirs() }
                    val name = queryDisplayName(uri) ?: "picked.img"
                    val dest = File(dir, name)
                    ksuApp.contentResolver.openInputStream(uri)?.use { input ->
                        dest.outputStream().use { input.copyTo(it) }
                    } ?: error("打不开这个文件")
                    dest
                }
            }
            copied.fold(
                onSuccess = { file ->
                    _uiState.update {
                        it.copy(pickedImage = file, pickedName = file.name, error = null)
                    }
                },
                onFailure = { e -> _uiState.update { it.copy(error = e.message ?: "读不了这个文件") } },
            )
        }
    }

    fun clearPickedImage() {
        _uiState.update { it.copy(pickedImage = null, pickedName = "") }
    }

    fun setBackupFirst(on: Boolean) {
        _uiState.update { it.copy(backupFirst = on) }
    }

    /** 开始刷。 */
    fun flash() {
        val state = _uiState.value
        if (!state.hasRoot) {
            _uiState.update { it.copy(error = "没有 root，写不了分区", phase = InstallPhase.Failed) }
            return
        }
        if (!state.hasRecoveryPartition) {
            _uiState.update {
                it.copy(
                    error = "这台机器没有独立的 recovery 分区（A/B 机型把 recovery 合在 boot 里），刷不了",
                    phase = InstallPhase.Failed,
                )
            }
            return
        }
        viewModelScope.launch {
            _uiState.update {
                it.copy(phase = InstallPhase.Downloading, progress = 0, status = "准备中", error = null)
            }

            // 1. 拿到镜像：用户选的那个优先。
            val image: File? = state.pickedImage ?: run {
                val release = state.release
                if (release == null || release.downloadUrl.isBlank()) {
                    _uiState.update {
                        it.copy(
                            phase = InstallPhase.Failed,
                            error = "没有可下载的刷机包，也没有选镜像",
                        )
                    }
                    return@launch
                }
                withContext(Dispatchers.IO) { downloadRelease(release) }.fold(
                    onSuccess = { it },
                    onFailure = { e ->
                        _uiState.update {
                            it.copy(phase = InstallPhase.Failed, error = e.message ?: "下载失败")
                        }
                        return@launch
                    },
                )
            }

            // 2. 如果是 zip，解出里面的 img。
            val resolved = withContext(Dispatchers.IO) {
                if (image.name.endsWith(".zip", ignoreCase = true)) {
                    _uiState.update { it.copy(phase = InstallPhase.Extracting, status = "解出镜像") }
                    RecoveryFlasher.extractImageFromZip(
                        image,
                        File(ksuApp.cacheDir, "skyward-install"),
                    )
                } else {
                    Result.success(image)
                }
            }.fold(
                onSuccess = { it },
                onFailure = { e ->
                    _uiState.update {
                        it.copy(phase = InstallPhase.Failed, error = e.message ?: "解包失败")
                    }
                    return@launch
                },
            )

            // 3. 先备份。
            if (_uiState.value.backupFirst) {
                _uiState.update { it.copy(phase = InstallPhase.BackingUp, status = "备份当前 recovery") }
                withContext(Dispatchers.IO) {
                    RecoveryFlasher.backupCurrent(File(ksuApp.filesDir, "recovery-backup"))
                }.fold(
                    onSuccess = { file -> _uiState.update { it.copy(backupFile = file) } },
                    onFailure = { e ->
                        _uiState.update {
                            it.copy(
                                phase = InstallPhase.Failed,
                                error = "备份失败，已停下：${e.message}",
                            )
                        }
                        return@launch
                    },
                )
            }

            // 4. 写分区。
            _uiState.update { it.copy(phase = InstallPhase.Flashing, status = "写入 recovery 分区") }
            withContext(Dispatchers.IO) { RecoveryFlasher.flash(resolved) }.fold(
                onSuccess = {
                    _uiState.update {
                        it.copy(
                            phase = InstallPhase.Done,
                            progress = 100,
                            status = "已写入，重启到 recovery 生效",
                            error = null,
                        )
                    }
                },
                onFailure = { e ->
                    _uiState.update {
                        it.copy(phase = InstallPhase.Failed, error = e.message ?: "刷入失败")
                    }
                },
            )
        }
    }

    /**
     * 下载发布包到缓存目录。
     *
     * 走应用自己的 okhttp，和更新检查同一条路；大文件流式写盘，不整个读进
     * 内存——刷机包有四十多兆。
     */
    private fun downloadRelease(release: SkywardRelease): Result<File> = runCatching {
        val dir = File(ksuApp.cacheDir, "skyward-install").apply { mkdirs() }
        val dest = File(dir, release.assetName.ifBlank { "skyward.zip" })
        if (dest.isFile && dest.length() == release.sizeBytes && release.sizeBytes > 0) {
            return@runCatching dest
        }
        val request = okhttp3.Request.Builder().url(release.downloadUrl).build()
        ksuApp.okhttpClient.newCall(request).execute().use { response ->
            if (!response.isSuccessful) error("下载失败：HTTP ${response.code}")
            val body = response.body
            val total = body.contentLength().takeIf { it > 0 } ?: release.sizeBytes
            body.byteStream().use { input ->
                dest.outputStream().use { output ->
                    val buffer = ByteArray(64 * 1024)
                    var written = 0L
                    while (true) {
                        val read = input.read(buffer)
                        if (read <= 0) break
                        output.write(buffer, 0, read)
                        written += read
                        if (total > 0) {
                            val pct = (written * 100 / total).toInt().coerceIn(0, 100)
                            _uiState.update { it.copy(progress = pct, status = "下载中 $pct%") }
                        }
                    }
                }
            }
        }
        dest
    }

    private fun queryDisplayName(uri: Uri): String? = runCatching {
        ksuApp.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
            val index = cursor.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME)
            if (index >= 0 && cursor.moveToFirst()) cursor.getString(index) else null
        }
    }.getOrNull()
}
