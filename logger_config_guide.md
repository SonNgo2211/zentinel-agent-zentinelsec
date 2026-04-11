# Hướng Dẫn Cấu Hình Logger Định Tuyến (ZentinelSec Agent)

ZentinelSec Agent sử dụng module `WafAsyncLogger` đa luồng, cung cấp khả năng đẩy log (JSON Audit) tới đa dạng các kho chứa (Local File hoặc Remote Syslog UDP/TCP) với hiệu năng cao.

Dưới đây là các Biến môi trường (Environment Variables) hỗ trợ cấu hình Logger:

## 1. Local File Logging (Disk I/O)

Mặc định, Agent bật tính năng ghi file Audit Log để lưu lại các sự kiện WAF ngay trên ổ cứng của server.

| Biến Môi Trường | Giá trị mặc định | Diễn giải |
|-----------------|-----------------|----------|
| `ZENTINELSEC_ENABLE_FILE_LOG` | `true` | Cờ (boolean) quyết định việc Agent có tạo và ghi log vào ổ đĩa hay không. |
| `ZENTINELSEC_AUDIT_LOG_PATH` | `/var/log/zentinelsec_audit.json` | Đường dẫn tuyệt đối trỏ tới file JSON lưu trữ log. |
| `ZENTINELSEC_ENABLE_CONSOLE_LOG`| `true` | Cờ (boolean) quyết định việc In trực tiếp Audit Log JSON ra Console (thuận tiện cho `docker logs`). |

**Tắt ghi đĩa (Disable Disk I/O)**: Nếu bạn thiết kế hệ thống hoàn toàn Stateless, hãy vô hiệu hóa tính năng này:
```bash
export ZENTINELSEC_ENABLE_FILE_LOG=false
```

## 2. Remote Syslog Logging (Mạng)

Agent có khả năng Stream trực tiếp raw JSON payload sang một máy chủ Syslog từ xa mà không cần thông qua các công cụ thu thập log truyền thống như Fluentd/Logstash.

| Biến Môi Trường | Giá trị mặc định | Diễn giải |
|-----------------|-----------------|----------|
| `ZENTINELSEC_SYSLOG_URL` | *(None)* | URL của máy chủ Syslog đích. Hỗ trợ giao thức TCP (e.g. `tcp://10.0.0.5:514`) hoặc UDP (e.g. `udp://10.0.0.5:5143`). |

**Kích hoạt Remote Syslog**:
```bash
export ZENTINELSEC_SYSLOG_URL="udp://192.168.1.100:514"
```
Hệ thống sẽ đồng thời bắn JSON kèm header gốc `<134> WAF: ` về phía server chỉ định.

## Ví dụ cấu hình trong `docker-compose.yml`

Tận dụng mô hình đẩy qua cổng UDP của ELK / rsyslog và vô hiệu hóa lưu trữ nội tại:

```yaml
services:
  zentinelsec:
    image: whackers/zentinelsec-agent:v0.3.0-local
    environment:
      - ZENTINELSEC_ENABLE_FILE_LOG=false
      - ZENTINELSEC_SYSLOG_URL=udp://rsyslog-server:514
    # Không cần tạo Volume Mapping cho log nữa
```
