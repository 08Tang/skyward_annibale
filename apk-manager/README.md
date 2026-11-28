# Skyward Manager

Skyward Recovery 的配套应用。包名 `com.tkexig.skyward`。

它不是 SukiSU 的 Root 管理器，而是从 SukiSU-Nova 的 manager 改出来的一个
**配置前端**：Skyward Recovery 把它的设置写在 `/metadata/skyward/settings.json`，
这个应用读那份文件、改它、再写回去。快充档位、温控、亮度、灭屏时间、时区、
静态 IP 都在这一页里调，整份配置也能导出/导入成 JSON。

## 它不做什么

这是这个分支存在的理由，值得说清楚。

SukiSU 的 manager 之所以是个「Root 管理器」，靠的是一整套只有配套内核模块
才成立的东西：读内核里的授权名单、改 SELinux 规则、隐藏挂载、伪装版本、往
zygote 里注入、给每个应用单独配 profile、临时 root、越狱（late-load）模式。
这些能力全都经由 `libksud.so` 这个守护进程和内核里的 supercall 实现——也就
是说，它们本身就是「这台机器上装了 SukiSU」的指纹。

这个版本把这些连同指纹一起拿掉了：

- 删掉了 `libksud.so`（Rust 守护进程）和 `libkernelsu.so`（C++ JNI）。
  `Natives` 里 26 个 `external` 成员全部改成 Kotlin 实现或常量，APK 里没有
  一行原生代码，CI 也不再需要 NDK 和 Rust 工具链。
- 删掉了 KPM、SUSFS、内置 Zygisk、扩展、动态管理器、卸载管理器、应用配置
  （profile）、临时 root、越狱模式、内核版本伪装。
- 删掉了模块页、模块仓库页、超级用户页。
- 删掉了「随机化包名」脚本（`randomizer`）——那是专门用来躲检测的。

Root 只从系统那个 `su` 来：`Shell.Builder.build("su")`，需要全局挂载命名空间
时用 `su -mm`。没有任何一处去问内核「谁被授权了」。

## 保留了什么

- 首页、设置页、关于页
- 主题商店、配色、自定义外观、应用图标、背景裁剪、自定义文案——这些不碰
  root，通用
- 日志页（sulog 的读取部分）
- 新增的 **Skyward** 页

## 首页的两句话

首页状态卡片的文案按这个意思走：

| 显示 | 含义 |
| --- | --- |
| 工作中 | 已经拿到 root |
| 未授权Root | 还没拿到 root |

「工作中」在 SukiSU 里问的是「APK 和内核模块配对了吗」，这里改成「拿到 root
了吗」——没有内核模块可配对了。

## 版本号

`version.properties` 里一行 `versionName=1.0.0`。`versionCode` 由它算出来：
`major * 10000 + minor * 100 + patch`。

每次 CI 构建成功后把补丁号加 `0.0.1` 并提交回去，所以每个构建出来的包版本
号都不一样，也就能覆盖安装上一个。

## 签名

v1 + v2 + v3 全签，用的是固定的证书——同一把证书签出来的包才能互相覆盖
安装。**证书不进仓库**，CI 从 secret 还原。

本地构建想签名的话，照着 `sign.example.properties` 写一个
`sign.properties`（已在 `.gitignore` 里），或者直接用环境变量：

```sh
export KEYSTORE_FILE=/path/to/release.jks
export KEYSTORE_PASSWORD=...
export KEY_ALIAS=...
export KEY_PASSWORD=...
```

没给证书时构建照常进行，只是不签名。

## 构建

只能在 CI 里构建——这是项目的规矩，本地不编译、不跑 lint、不跑测试。

`.github/workflows/apk.yml` 干这些事：

1. 检出代码（`apk-manager/**` 有改动才触发）
2. 装 JDK 21 和 Android SDK（platform 37 / build-tools 37.0.0，**不装 NDK**）
3. 从 `APK_KEYSTORE_BASE64` 还原证书
4. `./gradlew :app:assembleRelease`
5. `apksigner verify --verbose` 确认三种签名都在
6. 上传 APK 为 artifact
7. 把 `version.properties` 的补丁号加 0.0.1 并提交回去

需要的 secrets：

| Secret | 内容 |
| --- | --- |
| `APK_KEYSTORE_BASE64` | `.jks` 文件的 base64（`base64 -w0 release.jks`） |
| `APK_KEYSTORE_PASSWORD` | keystore 口令 |
| `APK_KEY_ALIAS` | 密钥别名 |
| `APK_KEY_PASSWORD` | 密钥口令 |

Rust 那边的 `build.yml` / `preview.yml` 都加了 `paths-ignore: ["apk-manager/**"]`，
两套 CI 互不触发。

## 配置文件的读写

`/metadata/skyward/settings.json` 由 Skyward 用 serde 序列化，路径在
`crates/skyward-ui/src/settings.rs`。有几点必须照着来：

- **先挂载再读写**。Skyward 平时让 `/metadata` 只读，启动完成后通常根本没
  挂载，直接读会读到空目录，误以为「没有配置」。`SkywardConfig` 读写前都先
  `mount /metadata`。
- **写回时每个字段都要带上**。Rust 那边有一半字段没有 `#[serde(default)]`，
  少一个 Skyward 就解析失败、退回默认值，用户的设置就丢了。Kotlin 这边靠
  `Json { encodeDefaults = true }` 保证。
- **读不出来的配置不覆盖**。一份解析失败的 JSON 可能是新版 Skyward 写的，
  直接盖掉就找不回来了，所以 `read()` 失败时只显示默认值，不写文件。
- **写入是原子的**。先写 `settings.json.new`，`chmod 600`，再 `mv -f` 过去。
  写一半断电留下的还是旧配置。

## 目录

```
apk-manager/
├── app/
│   └── src/main/java/com/tkexig/skyward/
│       ├── Natives.kt              # 原 JNI 层，现在是纯 Kotlin
│       ├── data/skyward/
│       │   ├── SkywardSettings.kt  # settings.json 的 Kotlin 镜像
│       │   ├── SkywardConfig.kt    # 读写 /metadata/skyward/settings.json
│       │   └── SuGrants.kt         # 授权记录（只是记录，不是授权本身）
│       ├── ui/screen/skyward/      # 新增的 Skyward 配置页
│       └── ui/util/KsuCli.kt       # 只剩开 shell、重启、读设备信息
├── version.properties
└── sign.example.properties
```

## 关于 `SuGrants`

`data/skyward/SuGrants.kt` 记着「哪些应用被授权过」，但它**不负责授权**——
真正的授权在系统那个 su 自己的界面里。这个文件只是把看到的结果记下来，给
首页和设置页显示用。这是刻意的：一旦由这个应用去写授权名单，它就又变回
Root 管理器了。
