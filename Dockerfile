# ==========================================
# 多阶段构建：Claude Code Rust 容器镜像
# ==========================================

# 阶段 1: 构建阶段
FROM rust:1.75 as builder

WORKDIR /build

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 复制源代码
COPY src ./src

# 构建优化版本
RUN cargo build --release

# ==========================================
# 阶段 2: 运行时阶段（最小镜像）
FROM alpine:3.18

LABEL maintainer="claude-code-rust"
LABEL description="High-performance Claude Code CLI - Rust Edition"
LABEL version="0.1.0"

# 安装必要的运行时依赖
RUN apk add --no-cache \
    ca-certificates \
    curl \
    libssl3 \
    libcrypto3

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /build/target/release/claude-code /usr/local/bin/

# 创建配置目录
RUN mkdir -p /home/claude/.config/claude-code

# 设置环境变量
ENV PATH="/usr/local/bin:${PATH}" \
    HOME="/home/claude" \
    XDG_CONFIG_HOME="/home/claude/.config"

# 创建非特权用户
RUN addgroup -D claude && \
    adduser -D -G claude claude && \
    chown -R claude:claude /home/claude

# 切换到非特权用户
USER claude

# 验证安装
RUN claude-code --version

# 设置入口点
ENTRYPOINT ["claude-code"]

# 默认命令：显示帮助
CMD ["--help"]

# ==========================================
# 构建说明：
# ==========================================
# docker build -t claude-code-rust:latest .
# docker build -t claude-code-rust:0.1.0 .
#
# 运行数据卷挂载：
# docker run -it --rm -v ~/.config/claude-code:/home/claude/.config/claude-code claude-code-rust
# 
# 使用示例：
# docker run --rm claude-code-rust --version
# docker run -it --rm claude-code-rust repl
# docker run --rm claude-code-rust query --prompt "What is Rust?"
