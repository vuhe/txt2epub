# txt2epub

## 安装

软件需要下载源码自行编译、构建、安装，软件**暂时没有**发布可执行文件或发布到 `crates.io` 的任何计划

在下载完成后，需用使用 rust 的 cargo 工具执行安装，安装 cargo 参见[文档](https://www.rust-lang.org/zh-CN/tools/install)

切换终端工作目录到源码目录，并运行安装命令：

```shell
cargo install --path .
```

## 已知问题

> 已知问题不会解决，此程序运行环境为 unix，windows 等系统暂时没有支持计划

### 文件编码

程序会假定输入的文件名和文件内容为 utf-8 编码

如果文件名或内容使用其他编码需要先转换为符合 utf-8 的编码

由于部分系统，例如：windows，中文件名可能含有部分无法识别的字符，在运行前需要确保文件名不存在非法字符

转换文件内容在 unix like 下可以使用 `enca` 命令转换为 utf-8:

```shell
enca -L zh_CN -x UTF-8 *
```

### 文件路径

程序使用的 epub 标准路径分隔符为 `/`，在 unix like 系统中可以正确处理

但在其他系统中，例如：windows 中分隔符使用 `\\`，这可能导致生成的文件出现路径错误

出现此问题的原因主要为 epub 内部统一使用 `/` 作为文件路径，在使用其他字符分割符时，可能识别错误

例如 windows 中的 `test\a.txt` 路径会被 epub 内部当作名称为 `test\a` 的 txt 文件
