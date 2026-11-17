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
- GitHub Actions 仅构建 `annibale`
- Recovery 在线更新指向本仓库的 Release

功能和日志已在 REDMI K90、`OS4.0.6.0.XPKCNXM` 上验证。Recovery 仍有风险，
刷入前请确认机型与系统版本一致，并备份重要数据。

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

本项目继续遵循 [GPL-3.0-or-later](LICENSE) 许可证。

---

© 2026 TKexig & Kirk Lin
