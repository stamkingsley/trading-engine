.PHONY: build test bench clean setup start stop status logs docker-build docker-run

# 默认目标
all: setup build

# 设置环境
setup:
	@echo "Setting up development environment..."
	@./scripts/setup.sh

# 构建项目
build:
	@echo "Building trading engine..."
	@RUSTFLAGS="-C target-cpu=native" cargo build --release

# 运行测试
test:
	@echo "Running tests..."
	@cargo test

# 运行基准测试
bench:
	@echo "Running benchmarks..."
	@cargo bench

# 清理构建文件
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf logs/*
	@rm -rf target/criterion

# 启动服务
start:
	@./scripts/start.sh

# 停止服务
stop:
	@./scripts/monitor.sh stop

# 查看状态
status:
	@./scripts/monitor.sh status

# 查看日志
logs:
	@./scripts/monitor.sh logs

# 重启服务
restart:
	@./scripts/monitor.sh restart

# Docker构建
docker-build:
	@echo "Building Docker image..."
	@docker build -t trading-engine:latest .

# Docker运行
docker-run:
	@echo "Running Docker container..."
	@docker-compose up -d

# 格式化代码
fmt:
	@cargo fmt

# 代码检查
clippy:
	@cargo clippy -- -D warnings

# 代码覆盖率
coverage:
	@cargo tarpaulin --out Html --output-dir coverage

# 发布版本
release: clean fmt clippy test bench
	@echo "Creating release build..."
	@cargo build --release
	@echo "Release ready!"

# 开发环境
dev:
	@cargo watch -x 'run'

# 性能分析
profile:
	@echo "Running performance profiling..."
	@cargo build --release
	@perf record -g ./target/release/trading-engine --benchmark --duration 30
	@perf report