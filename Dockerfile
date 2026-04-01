# ==========================================
# 多阶段构建：Claude Code Rust 容器镜像
# ==========================================

# 阶段 1: 构建阶段 (使用 musl 静态链接，兼容 Alpine)
FROM rust:1.82-bookworm AS builder

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build

# 依赖缓存层
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm -rf src

# 复制源代码并构建
COPY src ./src
RUN touch src/main.rs && cargo build --release --target x86_64-unknown-linux-musl

# ==========================================
# 阶段 2: 运行时阶段（最小镜像）
FROM alpine:3.20

LABEL maintainer="claude-code-rust"
LABEL description="High-performance Claude Code CLI - Rust Edition"
LABEL version="0.1.0"

# 安装必要的运行时依赖
RUN apk add --no-cache ca-certificates curl

WORKDIR /app

# 从构建阶段复制静态链接的二进制文件
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/claude-code /usr/local/bin/

# 创建配置目录和非特权用户
RUN mkdir -p /home/claude/.config/claude-code && \
    addgroup -D claude && \
    adduser -D -G claude claude && \
    chown -R claude:claude /home/claude

# 设置环境变量
ENV PATH="/usr/local/bin:${PATH}" \
    HOME="/home/claude" \
    XDG_CONFIG_HOME="/home/claude/.config"

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
# 运行：
# docker run -it --rm claude-code-rust repl
# docker run --rm claude-code-rust --version
# docker run -it --rm -v ~/.claude-code:/home/claude/.config/claude-code claude-code-rust
