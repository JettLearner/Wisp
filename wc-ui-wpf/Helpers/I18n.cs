using System.Collections.Generic;
using System.IO;
using System.Text.Json;

namespace WcUiWpf.Helpers;

public static class I18n
{
    private static Dictionary<string, string> _strings = new();
    private static string _lang = "zh-CN";

    public static event EventHandler? LanguageChanged;

    public static string CurrentLang => _lang;

    public static void Init(string lang)
    {
        _lang = lang;
        Load();
    }

    public static void SetLanguage(string lang)
    {
        _lang = lang;
        Load();
        LanguageChanged?.Invoke(null, EventArgs.Empty);
    }

    private static void Load()
    {
        try
        {
            // 优先从程序目录加载，其次从内置资源
            var path = Path.Combine(AppContext.BaseDirectory, "i18n", $"{_lang}.json");
            if (File.Exists(path))
            {
                var json = File.ReadAllText(path);
                _strings = JsonSerializer.Deserialize<Dictionary<string, string>>(json) ?? new();
                return;
            }
        }
        catch { }

        // 内置默认翻译
        _strings = GetBuiltin(_lang);
    }

    public static string Get(string key, string? fallback = null)
    {
        if (_strings.TryGetValue(key, out var val))
            return val;
        return fallback ?? key;
    }

    private static Dictionary<string, string> GetBuiltin(string lang)
    {
        var zh = new Dictionary<string, string>
        {
            ["app.name"] = "Wallpaper Connecter",
            ["ball.tooltip"] = "点击打开 AI 对话",
            ["ball.settings"] = "Settings",
            ["ball.music"] = "音乐",
            ["ball.exit"] = "退出",
            ["chat.title"] = "AI 对话",
            ["chat.status.ready"] = "就绪",
            ["chat.status.connecting"] = "连接中...",
            ["chat.input.placeholder"] = "输入消息，Enter 发送，Shift+Enter 换行...",
            ["chat.new_session"] = "新对话",
            ["settings.title"] = "Settings",
            ["settings.appearance"] = "外观",
            ["settings.floating_ball"] = "悬浮球",
            ["settings.general"] = "通用",
            ["settings.about"] = "About",
            ["settings.opacity"] = "窗口透明度",
            ["settings.language"] = "语言",
            ["settings.confirm"] = "OK",
            ["settings.cancel"] = "Cancel",
            ["settings.restore_default"] = "恢复默认",
            ["settings.unsaved_title"] = "未保存的更改",
            ["settings.unsaved_msg"] = "有未保存的更改，是否保存？",
            ["settings.save"] = "保存",
            ["settings.dont_save"] = "不保存",
            ["music.not_playing"] = "未在播放",
            ["music.unknown_artist"] = "—",
            ["ai.suspected"] = "疑似 AI",
        };

        var en = new Dictionary<string, string>
        {
            ["app.name"] = "Wallpaper Connecter",
            ["ball.tooltip"] = "Click to open AI chat",
            ["ball.settings"] = "Settings",
            ["ball.music"] = "Music",
            ["ball.exit"] = "Exit",
            ["chat.title"] = "AI Chat",
            ["chat.status.ready"] = "Ready",
            ["chat.status.connecting"] = "Connecting...",
            ["chat.input.placeholder"] = "Type a message, Enter to send, Shift+Enter for newline...",
            ["chat.new_session"] = "New Chat",
            ["settings.title"] = "Settings",
            ["settings.appearance"] = "Appearance",
            ["settings.floating_ball"] = "Floating Ball",
            ["settings.general"] = "General",
            ["settings.about"] = "About",
            ["settings.opacity"] = "Window Opacity",
            ["settings.language"] = "Language",
            ["settings.confirm"] = "OK",
            ["settings.cancel"] = "Cancel",
            ["settings.restore_default"] = "Restore Defaults",
            ["settings.unsaved_title"] = "Unsaved Changes",
            ["settings.unsaved_msg"] = "You have unsaved changes. Save them?",
            ["settings.save"] = "Save",
            ["settings.dont_save"] = "Don't Save",
            ["music.not_playing"] = "Not Playing",
            ["music.unknown_artist"] = "—",
            ["ai.suspected"] = "Suspected AI",
        };

        return lang == "en-US" ? en : zh;
    }
}
