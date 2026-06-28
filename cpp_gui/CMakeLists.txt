cmake_minimum_required(VERSION 3.20)
project(LiziEngine2D_CPP_GUI LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# vcpkg integration:
# - 需要在 CMake GUI/命令行里指定 CMAKE_TOOLCHAIN_FILE 指向 vcpkg.cmake
#   示例：D:\vcpkg\scripts\buildsystems\vcpkg.cmake
if(NOT DEFINED CMAKE_TOOLCHAIN_FILE)
  message(FATAL_ERROR
    "CMAKE_TOOLCHAIN_FILE 未指定：必须使用 vcpkg toolchain 才能找到 glfw3。"
    "\n请在 CMake GUI 的 'Specify toolchain file' 填入：<你的vcpkg路径>\\scripts\\buildsystems\\vcpkg.cmake"
  )
endif()

# OpenGL（不同平台/包名可能导致 target 不同）
find_package(OpenGL REQUIRED)

# GLFW via vcpkg（不同版本可能暴露不同 package 名称/target 名称）
# 这里做兼容：优先 glfw3，再回退 glfw
find_package(glfw3 CONFIG QUIET)
if (NOT glfw3_FOUND)
  find_package(glfw CONFIG QUIET)
endif()
if (NOT glfw3_FOUND)
  if (NOT glfw_FOUND)
    message(FATAL_ERROR "未找到 GLFW 包。请确认 vcpkg toolchain 已配置，并已安装 glfw3/glfw。")
  endif()
endif()

add_executable(lizi2d_gui
  src/main.cpp
  src/FrameFileIO.cpp
)

target_include_directories(lizi2d_gui PRIVATE
  "${CMAKE_CURRENT_SOURCE_DIR}/src"
)

# 链接 glfw：不同包会提供不同 target
set(GLFW_LINK_TARGET "")
if (TARGET glfw)
  set(GLFW_LINK_TARGET glfw)
elseif (TARGET glfw3::glfw)
  set(GLFW_LINK_TARGET glfw3::glfw)
elseif (TARGET glfw3)
  set(GLFW_LINK_TARGET glfw3)
else()
  # 最终兜底：如果你只想快速定位问题，这里让 CMake 报错更明确
  message(FATAL_ERROR "找到了 GLFW 包，但未找到可链接的 target（期望 glfw / glfw3::glfw / glfw3 之一）。")
endif()

# 链接 OpenGL：OpenGL::GL 在某些配置里不存在
set(OPENGL_LINK_TARGET "")
if (TARGET OpenGL::GL)
  set(OPENGL_LINK_TARGET OpenGL::GL)
elseif (TARGET OpenGL::OpenGL)
  set(OPENGL_LINK_TARGET OpenGL::OpenGL)
else()
  message(FATAL_ERROR "未找到可链接的 OpenGL target（期望 OpenGL::GL 或 OpenGL::OpenGL）。")
endif()

target_link_libraries(lizi2d_gui PRIVATE
  ${GLFW_LINK_TARGET}
  ${OPENGL_LINK_TARGET}
)

# Windows: 静态运行时/后续可按需调整
if(MSVC)
  target_compile_options(lizi2d_gui PRIVATE /W4)
endif()
