package com.tkexig.skyward.data.model

/**
 * 模块仓库的排序方式。
 *
 * 仓库页这一版不在了，但设置里存着这个偏好，读设置的地方还在用它校验存下来
 * 的值，所以留着；排序本身（要碰 RepoModule）随仓库页一起删了。
 */
enum class RepoSort {
    UPDATED,
    NAME,
    STARS;

    companion object {
        /**
         * 读回存下的顺序。新版本存枚举名；更早的版本存的是 `UPDATED, CREATED,
         * NAME, STARS` 的序号，这里一并迁移（CREATED 已经没有了，落回 UPDATED）。
         */
        fun fromStored(name: String?, legacyOrdinal: Int?): RepoSort {
            name?.let { n -> entries.firstOrNull { it.name == n }?.let { return it } }
            return when (legacyOrdinal) {
                2 -> NAME
                3 -> STARS
                else -> UPDATED
            }
        }
    }
}
