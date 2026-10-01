using System.IO;
using System.Windows;
using System.Windows.Threading;
using WcUiWpf.Helpers;
using WcUiWpf.Windows;

namespace WcUiWpf;

public partial class App : Application
{
    public static WsClient? Core { get; private set; }
    public static FloatingBallWindow? Ball { get; private set; }
    public static SettingsWindow? Settings { get; private set; }
    public static MusicWindow? Music { get; private set; }

    private static readonly string LogDir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
        "wallpaper-connecter");
    private static readonly string LogPath = Path.Combine(LogDir, "ui.log");

    private int _port;
    private string _token = "";

    public static void Log(string msg)
    {
        try
        {
            Directory.CreateDirectory(LogDir);
            File.AppendAllText(LogPath, $"[{DateTime.Now:HH:mm:ss}] {msg}\n");
        }
        catch { }
    }

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        DispatcherUnhandledException += (s, ex) =>
        {
            Log($"[Dispatcher Exception] {ex.Exception}");
            ex.Handled = true;
        };
        AppDomain.CurrentDomain.UnhandledException += (s, ex) =>
        {
            Log($"[Unhandled Exception] {ex.ExceptionObject}");
        };
        TaskScheduler.UnobservedTaskException += (s, ex) =>
        {
            Log($"[Unobserved Task Exception] {ex.Exception}");
            ex.SetObserved();
        };

        try
        {
            Log("=== App starting ===");

            var args = e.Args;
            for (int i = 0; i < args.Length; i++)
            {
                if (args[i] == "--port" && i + 1 < args.Length)
                    _port = int.Parse(args[i + 1]);
                else if (args[i] == "--token" && i + 1 < args.Length)
                    _token = args[i + 1];
            }
            Log($"Args: port={_port}, token={(_token.Length > 0 ? "set" : "empty")}");

            if (_port == 0)
            {
                var cfg = ConfigManager.Load();
                _port = cfg.LastWsPort;
                _token = cfg.LastWsToken ?? "";
                Log($"Fallback config: port={_port}");
            }

            I18n.Init(ConfigManager.Load().Language);

            if (_port > 0)
            {
                Core = new WsClient(_port, _token);
                Core.Connected += OnCoreConnected;
                Core.Disconnected += OnCoreDisconnected;
                _ = Core.ConnectAsync();
                Log("WsClient started");
            }
            else
            {
                Log("No port, starting in offline mode");
            }

            _ = Task.Run(async () =>
            {
                await Task.Delay(3000);
                Dispatcher.Invoke(() =>
                {
                    if (Ball == null)
                    {
                        Log("Timeout: showing ball in offline mode");
                        ShowBall();
                    }
                });
            });
        }
        catch (Exception ex)
        {
            Log($"[Startup Exception] {ex}");
            ShowBall();
        }
    }

    private void ShowBall()
    {
        try
        {
            Ball = new FloatingBallWindow();
            Ball.Show();
            Log("FloatingBallWindow shown");
        }
        catch (Exception ex)
        {
            Log($"[ShowBall Exception] {ex}");
        }
    }

    private void OnCoreConnected(object? sender, EventArgs e)
    {
        Log("Core connected");
        Dispatcher.Invoke(() =>
        {
            if (Ball == null) ShowBall();
        });
    }

    private void OnCoreDisconnected(object? sender, EventArgs e)
    {
        Log("Core disconnected");
        Dispatcher.Invoke(() =>
        {
            try { ConfigManager.Save(ConfigManager.Load()); } catch { }
            Shutdown();
        });
    }

    public static void ShowSettings()
    {
        if (Settings == null)
        {
            Settings = new SettingsWindow();
            Settings.Closed += (s, e) => Settings = null;
        }
        Settings.Show();
        Settings.Activate();
    }

    public static void ToggleMusic()
    {
        if (Music == null)
        {
            Music = new MusicWindow();
            Music.Closed += (s, e) => Music = null;
            Music.Show();
        }
        else
        {
            if (Music.IsVisible) Music.Hide();
            else Music.Show();
        }
    }
}
