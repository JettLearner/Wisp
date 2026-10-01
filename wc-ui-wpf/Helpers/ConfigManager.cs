using System.IO;
using System.Text.Json;

namespace WcUiWpf.Helpers;

public class AppConfig
{
    public string Language { get; set; } = "zh-CN";
    public double WindowOpacity { get; set; } = 0.85;
    public double BallOpacity { get; set; } = 0.85;
    public int MusicWindowX { get; set; } = -1;
    public int MusicWindowY { get; set; } = -1;
    public int LastWsPort { get; set; } = 0;
    public string? LastWsToken { get; set; }
}

public static class ConfigManager
{
    private static readonly string ConfigDir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
        "wallpaper-connecter");
    private static readonly string ConfigPath = Path.Combine(ConfigDir, "ui-config.json");

    public static AppConfig Load()
    {
        try
        {
            if (File.Exists(ConfigPath))
            {
                var json = File.ReadAllText(ConfigPath);
                return JsonSerializer.Deserialize<AppConfig>(json) ?? new AppConfig();
            }
        }
        catch { }
        return new AppConfig();
    }

    public static void Save(AppConfig config)
    {
        try
        {
            Directory.CreateDirectory(ConfigDir);
            var json = JsonSerializer.Serialize(config, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(ConfigPath, json);
        }
        catch { }
    }
}
