using System.Net.WebSockets;
using System.Text;
using System.Text.Json;

namespace WcUiWpf.Helpers;

public class WsEvent
{
    public string Type { get; set; } = "";
    public JsonElement Data { get; set; }
}

public class WsClient
{
    private readonly int _port;
    private readonly string _token;
    private ClientWebSocket? _ws;
    private CancellationTokenSource? _cts;
    private bool _running;
    private ulong _pingSeq;

    public event EventHandler? Connected;
    public event EventHandler? Disconnected;
    public event EventHandler<WsEvent>? EventReceived;

    public WsClient(int port, string token)
    {
        _port = port;
        _token = token;
    }

    public async Task ConnectAsync()
    {
        _cts = new CancellationTokenSource();
        _running = true;

        while (_running)
        {
            try
            {
                _ws = new ClientWebSocket();
                var uri = new Uri($"ws://127.0.0.1:{_port}/");
                await _ws.ConnectAsync(uri, _cts.Token);

                // 认证：core 期望第一条消息是纯 token 字符串
                await SendRawAsync(_token);
                App.Log("[Ws] Auth token sent");

                Connected?.Invoke(this, EventArgs.Empty);

                // 启动心跳和接收
                var heartbeat = HeartbeatLoopAsync();
                await ReceiveLoopAsync();
            }
            catch (Exception)
            {
                await Task.Delay(1000);
            }
            finally
            {
                Disconnected?.Invoke(this, EventArgs.Empty);
                _ws?.Dispose();
                _ws = null;
            }
        }
    }

    private async Task ReceiveLoopAsync()
    {
        var buffer = new byte[65536];
        while (_ws != null && _ws.State == WebSocketState.Open && !_cts!.IsCancellationRequested)
        {
            var result = await _ws.ReceiveAsync(new ArraySegment<byte>(buffer), _cts.Token);
            if (result.MessageType == WebSocketMessageType.Text)
            {
                var json = Encoding.UTF8.GetString(buffer, 0, result.Count);
                App.Log($"[Ws] Recv: {json.Substring(0, Math.Min(json.Length, 300))}");
                try
                {
                    using var doc = JsonDocument.Parse(json);
                    var root = doc.RootElement;

                    // Rust Event 格式: {"id":"...","source":"core","payload":{"type":"ping","data":{"seq":1}}}
                    if (root.TryGetProperty("payload", out var payload) && payload.ValueKind == JsonValueKind.Object)
                    {
                        var type = payload.TryGetProperty("type", out var t) ? t.GetString() ?? "" : "";
                        var data = payload.TryGetProperty("data", out var d) ? d : default;

                        // 收到 Ping 自动回 Pong（core 靠这个判断 UI 存活）
                        if (type == "ping")
                        {
                            var seq = data.TryGetProperty("seq", out var s) ? s.GetUInt64() : 0;
                            App.Log($"[Ws] Received Ping seq={seq}, sending Pong");
                            _ = SendEventAsync("pong", new { seq });
                        }
                        if (type == "pong")
                        {
                            App.Log($"[Ws] Received Pong");
                        }

                        EventReceived?.Invoke(this, new WsEvent { Type = type, Data = data });
                    }
                }
                catch (Exception ex)
                {
                    App.Log($"[Ws] Parse error: {ex.Message}");
                }
            }
        }
    }

    private async Task HeartbeatLoopAsync()
    {
        while (_ws != null && _ws.State == WebSocketState.Open && !_cts!.IsCancellationRequested)
        {
            try
            {
                _pingSeq++;
                App.Log($"[Ws] Sending Ping seq={_pingSeq}");
                await SendEventAsync("ping", new { seq = _pingSeq });
            }
            catch { break; }
            await Task.Delay(500);
        }
    }

    /// <summary>
    /// 发送 Rust Event 格式的 JSON（serde internally tagged: {"type":"...","data":...}）
    /// </summary>
    public async Task SendEventAsync(string type, object? data = null)
    {
        var evt = new
        {
            id = Guid.NewGuid().ToString(),
            timestamp = DateTime.UtcNow.ToString("o"),
            source = "ui",
            payload = data != null
                ? new { type, data }
                : new { type, data = (object?)null }
        };
        var json = JsonSerializer.Serialize(evt);
        await SendRawAsync(json);
    }

    private async Task SendRawAsync(string message)
    {
        if (_ws == null || _ws.State != WebSocketState.Open) return;
        var bytes = Encoding.UTF8.GetBytes(message);
        await _ws.SendAsync(new ArraySegment<byte>(bytes), WebSocketMessageType.Text, true, _cts!.Token);
    }

    public void Stop()
    {
        _running = false;
        _cts?.Cancel();
    }
}
