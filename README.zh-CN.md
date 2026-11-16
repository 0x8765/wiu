# wiu

[Français](README.md) · [English](README.en.md) · **简体中文** · [Español](README.es.md) · [Русский](README.ru.md)

使用 Rust 编写的 Windows x64 图片查看器。仅查看图片，不编辑或修改照片。

## 使用方法

从源码使用时，请先按照[编译](#编译)一节编译程序并生成 `dist/` 文件夹。`target/` 和 `dist/` 均为本地生成的文件夹，不纳入 Git 版本控制。

打开 `dist/wiu.exe`。程序以最大化窗口启动，保留 Windows 标题栏，并弹出对话框供您选择一张或多张图片。取消选择后，窗口仍可继续打开图片。程序也支持拖放图片。按 F11 进入全屏，按 F11 或 Esc 退出全屏。

```powershell
.\dist\wiu.exe "C:\Photos\summer 2026.jpg"
.\dist\wiu.exe "C:\Photos\a.png" "C:\Photos\b.tiff"
```

| 快捷键 | 功能 |
| --- | --- |
| Ctrl+O | 选择图片 |
| 左／右方向键 | 上一张／下一张图片 |
| 鼠标滚轮 | 以光标位置为中心缩放 |
| 按住鼠标左键拖动 | 平移图片 |
| 0 / Adapter | 适应窗口 |
| 1 / 100 % | 实际大小 |
| F11 / Esc | 进入／退出全屏 |
| 空格 | 暂停／继续播放 GIF |

打开单张图片时，可浏览同一文件夹中按名称排序的所有兼容图片。传入多个路径时，仅浏览所选图片。对于由 `image` 解码的格式，显示时会遵循 EXIF 方向信息。打开文件不会改变其任何字节。

## Windows 文件资源管理器集成

在程序窗口中点击 **Installer**，或运行：

```powershell
.\dist\wiu.exe --install
```

此操作会将 wiu 复制到 `%LOCALAPPDATA%\Programs\wiu\wiu.exe`，并为当前用户在“打开方式”中注册此程序，同时添加 **Ouvrir avec wiu** 右键菜单项。安装成功后，**Installer** 按钮会消失。如果启动的另一个可执行文件与已安装副本的内容不同，窗口会显示 **Mettre à jour** 按钮。更新完成后该按钮消失；两个副本内容相同时，不显示安装按钮。更新前请关闭其他 wiu 窗口。在 Windows 11 中，经典菜单项可能位于“显示更多选项”下。无需管理员权限。双击图片时使用的默认应用仍由用户在 Windows 中选择。

要移除这些注册项，请运行 `wiu.exe --uninstall`。随后关闭 wiu，如有需要，可删除其安装文件夹。可以重复安装；替换已安装的可执行文件前，请先关闭该副本，再从新版便携程序执行安装。

## 格式与限制

- JPEG/JPG/JFIF、PNG、BMP、TIFF、ICO、PNM/PPM/PGM/PBM 和 TGA：使用内置 Rust 解码器。
- GIF：使用 Windows 内置解码器，支持动画透明度、帧延时和帧处置方式。
- WebP 和 AVIF：通过 Windows 上可用的 WIC 编解码器读取第一帧。可能需要安装 Windows 的 WebP／AV1 图像扩展。缺少编解码器时，程序会显示明确错误。因此，未安装相应编解码器的 Windows 系统不保证支持这两种格式。
- SVG：通过 Windows 10/11 的原生 Direct2D 按文档尺寸渲染静态图像。Direct2D 不支持部分复杂 SVG 功能（滤镜、脚本、动画）。外部引用会被拒绝。
- 多页 TIFF：仅读取第一页。ICO：由解码器选择图像版本。不支持 HEIC、RAW 或高级色彩配置文件／HDR 管理。
- 文件大小上限为 128 MiB；解码后的像素数据或 GIF 全部帧的总数据量上限为 256 MiB。加载任务在界面线程之外依次执行，过期的请求会被忽略。

## 编译

需要 Rust 稳定版（界面至少需要 1.92）、`x86_64-pc-windows-gnu` 目标，以及已加入 PATH 的 MinGW（`gcc`、`windres`）。

```powershell
cargo build --release --locked
cargo test --locked
```

依赖版本锁定在 `Cargo.lock` 中。如果所有依赖均已缓存，可在 Cargo 命令中添加 `--offline`。界面使用 eframe/egui；对话框、GIF 解码和可选编解码器使用 Windows 功能；SVG 使用 Direct2D 软件渲染。运行时不会下载任何内容。

编译生成的可执行文件位于 `target/release/wiu.exe`。编译后，使用 Python 生成分发文件夹：

```powershell
python tools/package.py
```

脚本会将可执行文件、本说明的全部五种语言版本、项目许可证和依赖许可证复制到 `dist/`。脚本使用 `cargo metadata --offline`，因此所有依赖必须已缓存。

无需打开窗口即可检查解码：`wiu.exe --check "image.png" "image.svg"`（所有文件均可读取时退出码为 0，否则为 1）。测试样本生成在 `target/fixtures` 中。

注册表修改仅限于 `HKCU\Software\Classes\Applications\wiu.exe`、`HKCU\Software\Classes\wiu.Image`、`SystemFileAssociations` 下的 `wiu` 操作项、`HKCU\Software\wiu`，以及 `RegisteredApplications` 中的 `wiu` 值。

## 许可证

wiu 根据 [MIT 许可证](LICENSE)分发。依赖许可证包含在生成的分发文件夹中。
