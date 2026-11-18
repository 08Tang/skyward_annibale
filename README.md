![Skyward Recovery](assets/hero.jpg)

# Skyward Annibale

面向 **REDMI K90（`annibale`）** 的非官方 Skyward Recovery 适配，基于
[Kirk Lin 的 Skyward](https://github.com/kirklin/skyward) 开发。

当前适配基线为 `OS4.0.6.0.XPKCNXM`，对应版本标识
`OS4.0.6.0.WPKCNXM.D02`。本项目只支持并编译 `annibale`，其他机型不在适用范围内。

## 已适配

- Android 4.0 的 data 解密和 `/data` 挂载
- Wi-Fi 驱动、固件加载和连接
- 屏幕休眠唤醒后的触摸恢复
- 基于 `OS4.0.6.0` 原厂 recovery 构建卡刷镜像
- 基带分区备份，刷机后不会丢 IMEI

功能和日志已在 REDMI K90、`OS4.0.6.0.XPKCNXM` 上验证。Recovery 仍有风险，
刷入前请确认机型与系统版本一致，并备份重要数据。

## 功能

除 Skyward 本身的双系统、备份还原、文件管理、终端和分区工具外，本适配还包含：

- **模块管理**：查看、开关和卸载 KernelSU 与 Magisk 的模块
- **U 盘**：手机与 U 盘之间互传文件
- **无线传输**：把文件夹通过网页分享给同一网络下的电脑
- **从网址取文件**：输入链接直接下载，长链接可用二维码让另一台设备发来
- **文件排序与多选**：批量复制、移动、删除
- **账号头像**：显示手机自己的名字与系统版本
- **AVB**：查看并开关当前槽位的校验
- **更新状态**：读取 `/metadata` 里记录的 OTA 状态
- **记忆**：屏幕密码与 WLAN 会记住，下次自动连接

## 刷入

1. 解锁 Bootloader，并确认手机系统版本为 `OS4.0.6.0.XPKCNXM`。
2. 从本仓库的 Actions 或 Releases 下载 `annibale` 镜像压缩包。
3. 解压后进入 fastboot，刷入当前槽位的 recovery 分区。

```sh
unzip skyward-*-annibale-*.zip
fastboot flash recovery skyward-*-annibale-*.img
fastboot reboot recovery
```

请勿将此镜像刷入其他机型。

## 致谢

- 感谢 [Kirk Lin](https://github.com/kirklin) 创建并维护 Skyward。
- 感谢 [skyward-pudding](https://github.com/mariusarva/skyward-pudding) 项目
  （作者 [mariusarva](https://github.com/mariusarva)）在 Xiaomi 17 上的适配工作。
  本适配的模块管理、U 盘、无线传输、从网址取文件、文件排序与多选、账号头像、
  屏幕密码与 WLAN 记忆等功能，参考并移植自该项目。
- 感谢 Skyward 上游贡献者，以及设备适配、测试和文档贡献者。
- 感谢 Wi-Fi、data 解密和触摸问题相关 PR 的作者与参与者，包括
  [#30](https://github.com/kirklin/skyward/pull/30)、
  [#42](https://github.com/kirklin/skyward/pull/42)、
  [#48](https://github.com/kirklin/skyward/pull/48) 和
  [#60](https://github.com/kirklin/skyward/pull/60) 提供的修复思路与验证经验。

## 原项目

[Skyward](https://github.com/kirklin/skyward) 是使用 Rust 编写的开源 Recovery。
本仓库是非官方 `annibale` 分支，不是 Skyward 原项目，也不代表上游作者发布。
原项目代码、文档及相关资源的版权归原作者和贡献者所有。

本仓库另有部分功能移植自
[skyward-pudding](https://github.com/mariusarva/skyward-pudding)，
其版权归该项目作者和贡献者所有。

本项目继续遵循 [GPL-3.0-or-later](LICENSE) 许可证。

---

© 2026 Kirk Lin & TKexig
