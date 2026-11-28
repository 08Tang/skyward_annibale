package com.tkexig.skyward.ui.util

import android.net.Uri
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.withContext
import com.tkexig.skyward.data.registry.ExpectedFile
import com.tkexig.skyward.appContext
import com.tkexig.skyward.ui.util.module.LatestVersionInfo
import okhttp3.Request

/**
 * @author weishu
 * @date 2023/6/22.
 */
suspend fun download(
    url: String,
    fileName: String,
    onDownloaded: (Uri) -> Unit = {},
    onDownloading: () -> Unit = {},
    onProgress: (Int) -> Unit = {},
    expected: ExpectedFile? = null,
    onFailed: (DownloadManager.DownloadState) -> Unit = {},
) {
    onDownloading()

    val downloadId = DownloadManager.enqueue(
        context = appContext,
        url = url,
        fileName = fileName,
        onCompleted = onDownloaded,
        expected = expected,
    )

    val finalState = DownloadManager.downloads
        .onEach { map -> map[downloadId]?.let { onProgress(it.progress) } }
        .first { map ->
            val status = map[downloadId]?.status
            status == DownloadManager.Status.COMPLETED ||
                status == DownloadManager.Status.FAILED
        }[downloadId]
    if (finalState?.status == DownloadManager.Status.FAILED) {
        withContext(Dispatchers.Main) { onFailed(finalState) }
    }
}

internal suspend fun isDownloadAvailable(uri: Uri): Boolean = withContext(Dispatchers.IO) {
    runCatching {
        appContext.contentResolver.openFileDescriptor(uri, "r").use { it != null }
    }.getOrDefault(false)
}


/**
 * 这个应用自己的更新检查。
 *
 * 原来是查 SukiSU 的发布仓库，装上的会是别人的管理器——名字、包名、功能全
 * 对不上。改成查 Skyward 自己的仓库，只认带我们自己签名的 APK。
 *
 * 「CI 版」这个概念也没有了：这个应用没有内核模块要跟管理器配对，稳定版
 * 和测试版的区别只剩 tag。两个入口都指向同一个最新发布。
 */
fun checkNewVersion(): LatestVersionInfo = checkLatestRelease()

/** 保留旧入口，行为与 [checkNewVersion] 相同。 */
fun checkNewCiVersion(): LatestVersionInfo = LatestVersionInfo()

private const val RELEASE_API = "https://api.github.com/repos/08Tang/skyward_annibale/releases"

/**
 * 取最新一个正式发布里的 APK。
 *
 * 只认 `Skyward-*.apk` 这个名字；发布里还有别的附件（比如刷机包），不能随
 * 便挑一个装上去。
 */
private fun checkLatestRelease(): LatestVersionInfo {
    if (!isNetworkAvailable(appContext)) return LatestVersionInfo()
    return runCatching {
        val request = Request.Builder()
            .url("$RELEASE_API?per_page=5")
            .header("Accept", "application/vnd.github+json")
            .build()
        appContext.okhttpClient.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                android.util.Log.w("UpdateCheck", "HTTP ${response.code}")
                return LatestVersionInfo()
            }
            val releases = org.json.JSONArray(response.body.string())
            for (i in 0 until releases.length()) {
                val rel = releases.optJSONObject(i) ?: continue
                if (rel.optBoolean("draft") || rel.optBoolean("prerelease")) continue
                val assets = rel.optJSONArray("assets") ?: continue
                for (j in 0 until assets.length()) {
                    val asset = assets.optJSONObject(j) ?: continue
                    val name = asset.optString("name")
                    if (!name.startsWith("Skyward-") || !name.endsWith(".apk")) continue
                    // versionCode 从文件名里取：Skyward-<ver>-<code>.apk
                    val code = Regex("""-(\d+)\.apk$""").find(name)
                        ?.groupValues?.get(1)?.toLongOrNull() ?: continue
                    return LatestVersionInfo(
                        versionCode = code,
                        downloadUrl = asset.optString("browser_download_url"),
                        changelog = rel.optString("body"),
                    )
                }
            }
            LatestVersionInfo()
        }
    }.onFailure {
        android.util.Log.w("UpdateCheck", "update check failed", it)
    }.getOrDefault(LatestVersionInfo())
}
