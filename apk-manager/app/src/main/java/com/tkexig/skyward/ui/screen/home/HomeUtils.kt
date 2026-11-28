package com.tkexig.skyward.ui.screen.home

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.core.content.pm.PackageInfoCompat
import com.tkexig.skyward.Natives
import com.tkexig.skyward.Natives.isManager
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonArray

data class ManagerVersion(
    val versionName: String,
    val versionCode: Long
)

data class SystemInfo(
    val kernelVersion: String,
    val managerVersion: String,
    val deviceModel: String,
    val kernelFullVersion: String?,
    val fingerprint: String,
    val selinuxStatus: String,
    val seccompStatus: Int,
    val zygiskImplementation: String? = null,
)

fun getManagerVersion(context: Context): ManagerVersion {
    val packageInfo = context.packageManager.getPackageInfo(context.packageName, 0)!!
    val versionCode = PackageInfoCompat.getLongVersionCode(packageInfo)
    return ManagerVersion(
        versionName = packageInfo.versionName!!,
        versionCode = versionCode
    )
}




