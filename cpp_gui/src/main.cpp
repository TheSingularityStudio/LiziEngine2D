#include "FrameFileIO.h"

#if __has_include(<GLFW/glfw3.h>)
  #include <GLFW/glfw3.h>
#elif __has_include(<glfw3.h>)
  #include <glfw3.h>
#else
  #error "未找到 GLFW 头文件：请确认 vcpkg/glfw3 的 include 目录已配置到编译/IntelliSense。"
#endif

#include <array>
#include <chrono>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <string>
#include <thread>

static GLuint compile_shader(GLenum type, const char* src) {
    GLuint s = glCreateShader(type);
    glShaderSource(s, 1, &src, nullptr);
    glCompileShader(s);

    GLint ok = 0;
    glGetShaderiv(s, GL_COMPILE_STATUS, &ok);
    if (!ok) {
        GLint len = 0;
        glGetShaderiv(s, GL_INFO_LOG_LENGTH, &len);
        std::string msg(len, '\0');
        glGetShaderInfoLog(s, len, nullptr, msg.data());
        std::cerr << "shader compile failed: " << msg << std::endl;
        glDeleteShader(s);
        return 0;
    }
    return s;
}

static GLuint link_program(GLuint vs, GLuint fs) {
    GLuint p = glCreateProgram();
    glAttachShader(p, vs);
    glAttachShader(p, fs);
    glLinkProgram(p);

    GLint ok = 0;
    glGetProgramiv(p, GL_LINK_STATUS, &ok);
    if (!ok) {
        GLint len = 0;
        glGetProgramiv(p, GL_INFO_LOG_LENGTH, &len);
        std::string msg(len, '\0');
        glGetProgramInfoLog(p, len, nullptr, msg.data());
        std::cerr << "program link failed: " << msg << std::endl;
        glDeleteProgram(p);
        return 0;
    }
    return p;
}

int main() {
    if (!glfwInit()) {
        std::cerr << "glfwInit failed" << std::endl;
        return 1;
    }

    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

#ifdef __APPLE__
    glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
#endif

    const int win_w = 1000;
    const int win_h = 800;

    GLFWwindow* win = glfwCreateWindow(win_w, win_h, "LiziEngine2D - Step6 MVP (OpenGL) - V + Particles", nullptr, nullptr);
    if (!win) {
        std::cerr << "glfwCreateWindow failed" << std::endl;
        glfwTerminate();
        return 1;
    }
    glfwMakeContextCurrent(win);

    // OpenGL loader relies on platform; GLFW provides core functions on many setups.
    // For a robust solution, you’d add glad/glew; here we keep it minimal.

    const char* vs_src = R"(
        #version 330 core
        layout(location = 0) in vec2 aPos;

        out vec2 vUV;

        uniform vec2 uOffset;
        uniform vec2 uScale;

        void main() {
            vUV = (aPos * 0.5 + 0.5);
            vec2 p = uOffset + aPos * uScale;
            gl_Position = vec4(p, 0.0, 1.0);
        }
    )";

    const char* fs_src = R"(
        #version 330 core
        in vec2 vUV;
        out vec4 FragColor;

        uniform sampler2D uVTex; // GL_R32F

        uniform float uVMin;
        uniform float uVMax;

        vec3 heat(float t) {
            vec3 blue = vec3(0.2, 0.3, 0.9);
            vec3 white = vec3(1.0, 1.0, 1.0);
            vec3 red = vec3(0.9, 0.2, 0.2);

            if (t < 0.5) {
                float k = t / 0.5;
                return mix(blue, white, k);
            } else {
                float k = (t - 0.5) / 0.5;
                return mix(white, red, k);
            }
        }

        void main() {
            float v = texture(uVTex, vUV).r;
            float t = (v - uVMin) / max((uVMax - uVMin), 1e-12);
            t = clamp(t, 0.0, 1.0);
            vec3 c = heat(t);
            FragColor = vec4(c, 1.0);
        }
    )";

    GLuint vs = compile_shader(GL_VERTEX_SHADER, vs_src);
    GLuint fs = compile_shader(GL_FRAGMENT_SHADER, fs_src);
    if (!vs || !fs) return 1;

    GLuint program = link_program(vs, fs);
    glDeleteShader(vs);
    glDeleteShader(fs);

    if (!program) return 1;

    // 粒子 shader（GL_POINTS）
    const char* p_vs_src = R"(
        #version 330 core
        layout(location=0) in vec2 aPos;
        void main() { gl_Position = vec4(aPos, 0.0, 1.0); }
    )";

    const char* p_fs_src = R"(
        #version 330 core
        out vec4 FragColor;
        void main() { FragColor = vec4(0.0, 0.0, 0.0, 1.0); }
    )";

    GLuint pv = compile_shader(GL_VERTEX_SHADER, p_vs_src);
    GLuint pf = compile_shader(GL_FRAGMENT_SHADER, p_fs_src);
    if (!pv || !pf) return 1;

    GLuint particle_program = link_program(pv, pf);
    glDeleteShader(pv);
    glDeleteShader(pf);
    if (!particle_program) return 1;

    // 全屏四边形（NDC -1..1）
    float quad[8] = {
        -1.0f, -1.0f,
        +1.0f, -1.0f,
        -1.0f, +1.0f,
        +1.0f, +1.0f,
    };

    GLuint vao = 0, vbo = 0;
    glGenVertexArrays(1, &vao);
    glGenBuffers(1, &vbo);
    glBindVertexArray(vao);
    glBindBuffer(GL_ARRAY_BUFFER, vbo);
    glBufferData(GL_ARRAY_BUFFER, sizeof(quad), quad, GL_STATIC_DRAW);
    glEnableVertexAttribArray(0);
    glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 2 * sizeof(float), (void*)0);
    glBindVertexArray(0);

    // 粒子：单独 VBO（x,y 归一到 -1..1）
    GLuint pvao = 0, pvbo = 0;
    glGenVertexArrays(1, &pvao);
    glGenBuffers(1, &pvbo);

    glBindVertexArray(pvao);
    glBindBuffer(GL_ARRAY_BUFFER, pvbo);
    glBufferData(GL_ARRAY_BUFFER, 0, nullptr, GL_DYNAMIC_DRAW);
    glEnableVertexAttribArray(0);
    glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 2 * sizeof(float), (void*)0);
    glBindVertexArray(0);

    // V 纹理
    GLuint vtex = 0;
    glGenTextures(1, &vtex);
    glBindTexture(GL_TEXTURE_2D, vtex);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);
    glBindTexture(GL_TEXTURE_2D, 0);

    FrameFileIO fio;
    FrameSnapshot snap;
    std::string frame_path = "cpp_gui/frame_latest.bin";

    uint32_t nx = 0, ny = 0;

    bool has_frame = false;

    // 简单时间控制
    auto last_copy = std::chrono::steady_clock::now();

    while (!glfwWindowShouldClose(win)) {
        glfwPollEvents();

        // 轮询读取最新帧文件
        {
            bool ok = fio.load_latest(frame_path, snap);
            if (ok && snap.nx != 0 && snap.ny != 0) {
                // 若尺寸变化则重建纹理存储
                if (!has_frame || snap.nx != nx || snap.ny != ny) {
                    nx = snap.nx;
                    ny = snap.ny;
                    glBindTexture(GL_TEXTURE_2D, vtex);
                    glTexImage2D(
                        GL_TEXTURE_2D,
                        0,
                        GL_R32F,
                        (GLsizei)nx,
                        (GLsizei)ny,
                        0,
                        GL_RED,
                        GL_FLOAT,
                        nullptr
                    );
                    glBindTexture(GL_TEXTURE_2D, 0);
                }

                // 更新纹理数据（注意：V 在 Python 中通常是 (nx,ny)，这里按 (u,v) 采样，使用直接拷贝）
                glBindTexture(GL_TEXTURE_2D, vtex);
                glTexSubImage2D(
                    GL_TEXTURE_2D,
                    0,
                    0, 0,
                    (GLsizei)nx,
                    (GLsizei)ny,
                    GL_RED,
                    GL_FLOAT,
                    snap.V.data()
                );
                glBindTexture(GL_TEXTURE_2D, 0);

                // 更新粒子点 VBO：把 [0,L] wrap 后范围映射到 NDC -1..1
                // 注意：这里假设 Python 写入的 x,y 已经是 [0,1)*L 的物理坐标，
                // 但我们目前没有从文件携带 Lx/Ly，因此简化为假设 Lx=Ly=1。
                // 若你之后在文件头加 Lx/Ly，可在这里做精确映射。
                const float invL = 1.0f; // 假设 Lx=Ly=1
                std::vector<float> pts;
                pts.reserve(snap.n_particles * 2);
                for (uint32_t i = 0; i < snap.n_particles; i++) {
                    float px = snap.px[i] * invL;
                    float py = snap.py[i] * invL;
                    float ndc_x = px * 2.0f - 1.0f;
                    float ndc_y = py * 2.0f - 1.0f;
                    pts.push_back(ndc_x);
                    pts.push_back(ndc_y);
                }

                glBindBuffer(GL_ARRAY_BUFFER, pvbo);
                glBufferData(GL_ARRAY_BUFFER, (GLsizeiptr)(pts.size() * sizeof(float)), pts.data(), GL_DYNAMIC_DRAW);
                glBindBuffer(GL_ARRAY_BUFFER, 0);

                has_frame = true;
            }
        }

        glClearColor(0.1f, 0.1f, 0.12f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT);

        if (has_frame) {
            // Vmin/Vmax auto
            float vmin = snap.V.empty() ? -1.0f : snap.V[0];
            float vmax = snap.V.empty() ? 1.0f : snap.V[0];
            for (float v : snap.V) {
                vmin = std::min(vmin, v);
                vmax = std::max(vmax, v);
            }
            if (vmax == vmin) vmax = vmin + 1e-6f;

            glUseProgram(program);

            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, vtex);
            glUniform1i(glGetUniformLocation(program, "uVTex"), 0);
            glUniform1f(glGetUniformLocation(program, "uVMin"), vmin);
            glUniform1f(glGetUniformLocation(program, "uVMax"), vmax);

            // quad 填满（offset=0, scale=1）
            glUniform2f(glGetUniformLocation(program, "uOffset"), 0.0f, 0.0f);
            glUniform2f(glGetUniformLocation(program, "uScale"), 1.0f, 1.0f);

            glBindVertexArray(vao);
            glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);
            glBindVertexArray(0);

            glUseProgram(0);

            // 粒子绘制（GL_POINTS）
            glUseProgram(particle_program);
            glBindVertexArray(pvao);
            glPointSize(3.5f);
            glDrawArrays(GL_POINTS, 0, (GLsizei)snap.n_particles);
            glBindVertexArray(0);
            glUseProgram(0);
        }

        glfwSwapBuffers(win);

        // MVP 简单降低 CPU 占用
        std::this_thread::sleep_for(std::chrono::milliseconds(16));
    }

    glfwTerminate();
    return 0;
}
