# vcpkg 配置（GUI + CMake + glfw3）

目标：让 `cpp_gui` 能够通过 `find_package(glfw3 CONFIG REQUIRED)` 找到 GLFW 的头文件与库。

## 1) 确保已安装 vcpkg
- 确保你已经克隆/安装好 vcpkg（通常会有一个 `vcpkg.exe` 与 `scripts/buildsystems/vcpkg.cmake`）。

## 2) 安装依赖（通过 vcpkg）
在任意终端里（不依赖 CMake GUI）执行：
- `.\vcpkg.exe install glfw3`

> 如果你启用的是 vcpkg manifest（已存在 `cpp_gui/vcpkg.json`），也可以后续由 CMake 自动解析，但第一次通常建议你先 `install` 一次确认环境正常。

## 3) 用 CMake GUI 配置 Toolchain File（关键）
打开 **CMake GUI**：
1. `Where is the source code:` 填 `cpp_gui` 的上级目录（包含 `cpp_gui/CMakeLists.txt` 的目录）
2. `Where to build the binaries:` 选一个空目录（例如 `cpp_gui/build`）
3. `Specify toolchain file:` 填下面路径之一（取决于你 vcpkg 的实际安装目录）：

你提供的 vcpkg toolchain 路径是固定示例（请确认与自己机器一致）：
- `C:\Windows\System32\vcpkg\scripts\buildsystems\vcpkg.cmake`

4. 选好编译器（VS：MSVC；MinGW：对应 MinGW）
5. 点 `Configure`

如果 configure 成功，再点 `Generate`，最后 `Build`.

## 4) VSCode IntelliSense（可选，但推荐）
CMake GUI 配置正确后，VSCode 通常会从 CMake Tools/compile_commands 获取 include 路径。
如果 VSCode 仍然提示找不到 `GLFW/glfw3.h`：
- 确认你当前使用的编译配置与 CMake GUI 选用配置一致（Debug/Release）
- 确认 VSCode 的 C/C++ 扩展没有手动覆盖 IntelliSense includePath

---

## 5) 你当前工程的约定
- `cpp_gui/vcpkg.json` 已包含 `glfw3`
- `cpp_gui/CMakeLists.txt` 使用：
  - `find_package(glfw3 CONFIG REQUIRED)`
  - `target_link_libraries(lizi2d_gui PRIVATE glfw OpenGL::GL)`
- 因此只要 toolchain 配好，GLFW 的 include/库就会正确提供给编译器与链接器。
