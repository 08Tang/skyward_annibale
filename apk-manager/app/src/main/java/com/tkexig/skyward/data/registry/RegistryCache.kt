package com.tkexig.skyward.data.registry

import java.io.IOException

/**
 * 取一个 URL 的文本。
 *
 * 抽成接口是为了让 registry 的解析逻辑能脱离网络测试。
 */
fun interface RegistryFetcher {
    @Throws(IOException::class)
    fun fetch(url: String): String
}

/** registry 文档的缓存，按 key 存原文。 */
interface RegistryCache {
    fun read(key: String): String?
    fun write(key: String, value: String)
}
