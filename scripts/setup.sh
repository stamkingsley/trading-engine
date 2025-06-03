#!/bin/bash
set -e

echo "Setting up high-performance trading engine..."

# 检查依赖
check_dependencies() {
    echo "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        echo "Error: Rust/Cargo not found. Please install Rust first."
        exit 1
    fi
    
    if ! command -v flatc &> /dev/null; then
        echo "Installing FlatBuffers compiler..."
        # Ubuntu/Debian
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y flatbuffers-compiler
        # CentOS/RHEL
        elif command -v yum &> /dev/null; then
            sudo yum install -y flatbuffers-compiler
        # macOS
        elif command -v brew &> /dev/null; then
            brew install flatbuffers
        else
            echo "Error: Unable to install FlatBuffers. Please install manually."
            exit 1
        fi
    fi
}

# 系统优化
optimize_system() {
    echo "Applying system optimizations..."
    
    # 检查是否为root权限
    if [[ $EUID -eq 0 ]]; then
        # 设置CPU调度器
        echo "Setting CPU governor to performance..."
        for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
            if [ -f "$cpu" ]; then
                echo performance > "$cpu"
            fi
        done
        
        # 网络优化
        echo "Optimizing network settings..."
        sysctl -w net.core.rmem_max=134217728
        sysctl -w net.core.wmem_max=134217728
        sysctl -w net.ipv4.tcp_rmem="4096 87380 134217728"
        sysctl -w net.ipv4.tcp_wmem="4096 65536 134217728"
        
        # 内存优化
        echo "Optimizing memory settings..."
        echo always > /sys/kernel/mm/transparent_hugepage/enabled
        sysctl -w vm.overcommit_memory=1
        
        echo "System optimizations applied."
    else
        echo "Warning: Not running as root. Skipping system optimizations."
        echo "Run 'sudo ./scripts/setup.sh' for full optimization."
    fi
}

# 构建项目
build_project() {
    echo "Building trading engine..."
    
    # 清理之前的构建
    cargo clean
    
    # 构建发布版本
    RUSTFLAGS="-C target-cpu=native" cargo build --release
    
    echo "Build completed successfully."
}

# 创建必要的目录
create_directories() {
    echo "Creating directories..."
    mkdir -p logs
    mkdir -p data
    mkdir -p config
    chmod 755 logs data config
}

# 主函数
main() {
    check_dependencies
    create_directories
    optimize_system
    build_project
    
    echo ""
    echo "Setup completed successfully!"
    echo "You can now run the trading engine with:"
    echo "  ./scripts/start.sh"
    echo ""
    echo "Or run with custom configuration:"
    echo "  ./target/release/trading-engine --config config/production.toml"
}

main "$@"