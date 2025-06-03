#!/bin/bash

# 监控脚本
PID_FILE="/tmp/trading-engine/trading-engine.pid"

check_status() {
    if [[ -f "$PID_FILE" ]]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo "Trading engine is running (PID: $PID)"
            
            # 显示资源使用情况
            echo "Resource usage:"
            ps -p "$PID" -o pid,ppid,cmd,%cpu,%mem,vsz,rss,etime
            
            # 显示网络连接
            echo ""
            echo "Network connections:"
            netstat -tuln | grep ":8080\|:9090" || echo "No network connections found"
            
            return 0
        else
            echo "Trading engine is not running (stale PID file)"
            rm -f "$PID_FILE"
            return 1
        fi
    else
        echo "Trading engine is not running"
        return 1
    fi
}

stop_engine() {
    if [[ -f "$PID_FILE" ]]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo "Stopping trading engine (PID: $PID)..."
            kill -TERM "$PID"
            
            # 等待进程结束
            for i in {1..10}; do
                if ! ps -p "$PID" > /dev/null 2>&1; then
                    echo "Trading engine stopped successfully"
                    rm -f "$PID_FILE"
                    return 0
                fi
                sleep 1
            done
            
            # 强制终止
            echo "Force killing trading engine..."
            kill -KILL "$PID"
            rm -f "$PID_FILE"
        else
            echo "Trading engine is not running"
            rm -f "$PID_FILE"
        fi
    else
        echo "Trading engine is not running"
    fi
}

show_logs() {
    if [[ -f "logs/trading-engine.log" ]]; then
        tail -f logs/trading-engine.log
    else
        echo "Log file not found: logs/trading-engine.log"
    fi
}

case "$1" in
    status)
        check_status
        ;;
    stop)
        stop_engine
        ;;
    restart)
        stop_engine
        sleep 2
        ./scripts/start.sh
        ;;
    logs)
        show_logs
        ;;
    *)
        echo "Usage: $0 {status|stop|restart|logs}"
        echo ""
        echo "Commands:"
        echo "  status   - Show running status and resource usage"
        echo "  stop     - Stop the trading engine"
        echo "  restart  - Restart the trading engine"
        echo "  logs     - Show and follow log output"
        exit 1
        ;;
esac