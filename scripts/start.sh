#!/bin/bash
set -e

# 默认配置
BINARY_PATH="./target/release/trading-engine"
CONFIG_FILE="config/default.toml"
LOG_LEVEL="info"
WORKERS=""
NUMA_NODE=""

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        --log-level)
            LOG_LEVEL="$2"
            shift 2
            ;;
        --workers)
            WORKERS="$2"
            shift 2
            ;;
        --numa-node)
            NUMA_NODE="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --config FILE       Configuration file path"
            echo "  --log-level LEVEL   Log level (debug, info, warn, error)"
            echo "  --workers NUM       Number of worker threads"
            echo "  --numa-node NODE    NUMA node to bind to"
            echo "  --help              Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# 检查二进制文件是否存在
if [[ ! -f "$BINARY_PATH" ]]; then
    echo "Error: Binary not found at $BINARY_PATH"
    echo "Please run './scripts/setup.sh' first to build the project."
    exit 1
fi

# 设置环境变量
export RUST_LOG="$LOG_LEVEL"
export RUST_BACKTRACE=1

# 构建启动命令
CMD="$BINARY_PATH"

if [[ -n "$CONFIG_FILE" ]]; then
    CMD="$CMD --config $CONFIG_FILE"
fi

if [[ -n "$WORKERS" ]]; then
    CMD="$CMD --workers $WORKERS"
fi

# NUMA绑定
if [[ -n "$NUMA_NODE" ]]; then
    if command -v numactl &> /dev/null; then
        CMD="numactl --cpunodebind=$NUMA_NODE --membind=$NUMA_NODE $CMD"
    else
        echo "Warning: numactl not found, ignoring NUMA binding"
    fi
fi

# 创建PID文件目录
mkdir -p /tmp/trading-engine

# 启动服务
echo "Starting trading engine..."
echo "Command: $CMD"
echo "Log level: $LOG_LEVEL"
echo "Config file: $CONFIG_FILE"
echo ""

# 捕获信号以优雅关闭
trap 'echo "Shutting down trading engine..."; kill $PID; wait $PID' SIGINT SIGTERM

# 启动进程
$CMD &
PID=$!

echo "Trading engine started with PID: $PID"
echo $PID > /tmp/trading-engine/trading-engine.pid

# 等待进程结束
wait $PID
echo "Trading engine stopped."