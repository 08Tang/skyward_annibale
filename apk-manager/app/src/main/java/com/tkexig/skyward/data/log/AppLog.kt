package com.tkexig.skyward.data.log

import android.content.Context
import android.util.Log
import java.io.File
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/**
 * Skyward Manager 自己的运行日志。
 *
 * 只收这个应用说的话：它调了什么命令、拿到了什么结果、哪一步失败了。不收
 * 内核的 dmesg，也不收 ksud 的日志——这个应用已经不跟那些打交道了，把它们
 * 一起打包只会让报告变长、还带上别人的痕迹。
 *
 * 日志写在应用私有目录，只留最近 [MAX_BYTES] 字节：这东西是为了出问题时能
 * 看一眼，不是要长期归档。
 */
object AppLog {
    private const val TAG = "SkywardManager"
    private const val FILE_NAME = "skyward.log"
    private const val MAX_BYTES = 512L * 1024
    private const val MAX_LINES = 4000

    private val stamp = SimpleDateFormat("MM-dd HH:mm:ss.SSS", Locale.US)

    @Volatile
    private var file: File? = null

    /** 在 Application.onCreate 里调一次，之后写日志才有地方落。 */
    fun init(context: Context) {
        val dir = File(context.filesDir, "logs").apply { mkdirs() }
        file = File(dir, FILE_NAME)
    }

    fun d(message: String) = write("D", message, null)

    fun i(message: String) = write("I", message, null)

    fun w(message: String, error: Throwable? = null) = write("W", message, error)

    fun e(message: String, error: Throwable? = null) = write("E", message, error)

    /**
     * 写一行。
     *
     * 同时进 logcat 和文件：接电脑时看 logcat 更方便，出问题让人导日志时
     * 文件更方便。写文件失败就只留在 logcat，不能让记日志本身把功能弄挂。
     */
    @Synchronized
    private fun write(level: String, message: String, error: Throwable?) {
        when (level) {
            "E" -> Log.e(TAG, message, error)
            "W" -> Log.w(TAG, message, error)
            "D" -> Log.d(TAG, message, error)
            else -> Log.i(TAG, message, error)
        }
        val target = file ?: return
        runCatching {
            val line = buildString {
                append(stamp.format(Date()))
                append(' ')
                append(level)
                append(' ')
                append(message)
                if (error != null) {
                    append(" — ")
                    append(error.javaClass.simpleName)
                    append(": ")
                    append(error.message)
                }
                append('\n')
            }
            target.appendText(line)
            trimIfNeeded(target)
        }
    }

    /**
     * 超过上限就把前面一半丢掉。
     *
     * 不做精确的按字节截断：切在半个字符上会留下乱码。丢掉前面的行，留下的
     * 是最新的，这正是要看的部分。
     */
    private fun trimIfNeeded(target: File) {
        if (target.length() <= MAX_BYTES) return
        val lines = target.readLines()
        val kept = lines.takeLast(MAX_LINES / 2)
        target.writeText(kept.joinToString("\n", postfix = "\n"))
    }

    /** 现在这份日志的内容。 */
    fun read(): String {
        val target = file ?: return ""
        return runCatching { target.readText() }.getOrDefault("")
    }

    /** 日志文件本身，导出时用。 */
    fun file(): File? = file?.takeIf { it.isFile }

    fun clear() {
        runCatching { file?.writeText("") }
    }
}
