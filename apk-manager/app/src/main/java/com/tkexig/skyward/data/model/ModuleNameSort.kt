package com.tkexig.skyward.data.model

/**
 * 模块列表按名字排的方式。
 *
 * 模块页这一版不在了，但设置里存着这个偏好，读设置的地方还在用它校验
 * 存下来的值，所以留着。
 */
enum class ModuleNameSort {
    DEFAULT,
    ASCENDING,
    DESCENDING;

    companion object {
        fun fromStored(name: String?): ModuleNameSort =
            entries.firstOrNull { it.name == name } ?: DEFAULT
    }
}
