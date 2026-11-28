package com.tkexig.skyward.data.skyward

import android.util.Log
import com.tkexig.skyward.appContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.Request
import org.json.JSONObject

/**
 * Skyward 最新的一个发布。
 *
 * 安装页要显示版本号、文件大小、发布时间和更新内容，这些都是 GitHub 的
 * release 里现成的：tag_name、assets[].size、published_at、body。不用自己
 * 另存一份，免得和实际发布对不上。
 */
data class SkywardRelease(
    val tag: String,
    /** 发布包里那个 zip 的下载地址；没有 zip 的发布是空的。 */
    val downloadUrl: String = "",
    val assetName: String = "",
    val sizeBytes: Long = 0,
    /** 发布时间，ISO 8601，原样保留以便按 locale 显示。 */
    val publishedAt: String = "",
    /**
     * 更新内容，只取发布说明里「本次更新」那一节。
     *
     * 发布说明里还有适配信息和下载说明，那些是给刷机的人看的，不是这次改了
     * 什么。整段贴进安装页会把真正要看的东西淹掉。
     */
    val changelog: String = "",
)

/**
 * 从 GitHub 取 Skyward 的最新发布。
 *
 * 仓库是 annibale 分支对应的那个；用 releases 列表取第一个不是草稿、不是
 * 预发布的，因为安装页要给的是「现在该刷的那一版」。
 */
object SkywardReleases {
    private const val TAG = "SkywardReleases"
    private const val REPO = "08Tang/skyward_annibale"
    private const val API = "https://api.github.com/repos/$REPO/releases?per_page=10"

    suspend fun latest(): SkywardRelease? = withContext(Dispatchers.IO) {
        runCatching {
            appContext.okhttpClient.newCall(
                Request.Builder()
                    .url(API)
                    .header("Accept", "application/vnd.github+json")
                    .build()
            ).execute().use { response ->
                if (!response.isSuccessful) {
                    Log.w(TAG, "HTTP ${response.code}")
                    return@use null
                }
                val releases = org.json.JSONArray(response.body.string())
                for (i in 0 until releases.length()) {
                    val rel = releases.optJSONObject(i) ?: continue
                    if (rel.optBoolean("draft") || rel.optBoolean("prerelease")) continue
                    val zip = pickZip(rel) ?: continue
                    return@use SkywardRelease(
                        tag = rel.optString("tag_name"),
                        downloadUrl = zip.first,
                        assetName = zip.second,
                        sizeBytes = zip.third,
                        publishedAt = rel.optString("published_at"),
                        changelog = extractChangelog(rel.optString("body")),
                    )
                }
                null
            }
        }.onFailure { Log.w(TAG, "cannot read releases", it) }.getOrNull()
    }

    /**
     * 从发布说明里取出「本次更新」那一节。
     *
     * 找 `## 本次更新` 这个标题，取到下一个 `##` 标题为止。没有这一节就把
     * 整段正文当更新内容——那说明写说明的人没分节，正文就是全部内容。
     */
    private fun extractChangelog(body: String): String {
        if (body.isBlank()) return ""
        val lines = body.replace("\r\n", "\n").split('\n')
        val start = lines.indexOfFirst { it.trim().startsWith("##") && it.contains("本次更新") }
        if (start < 0) return body.trim()
        val end = lines.drop(start + 1)
            .indexOfFirst { it.trim().startsWith("## ") }
            .let { if (it < 0) lines.size else start + 1 + it }
        return lines.subList(start + 1, end).joinToString("\n").trim()
    }

    /**
     * 发布里的刷机包。
     *
     * 一个发布里可能同时有 SHA256SUMS 和几个 zip（按机型分），挑第一个 zip：
     * Skyward 的刷机包就是这个，校验文件不是。
     */
    private fun pickZip(rel: JSONObject): Triple<String, String, Long>? {
        val assets = rel.optJSONArray("assets") ?: return null
        for (i in 0 until assets.length()) {
            val a = assets.optJSONObject(i) ?: continue
            val name = a.optString("name")
            if (!name.endsWith(".zip", ignoreCase = true)) continue
            val url = a.optString("browser_download_url")
            if (url.isBlank()) continue
            return Triple(url, name, a.optLong("size"))
        }
        return null
    }
}
