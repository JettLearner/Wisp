using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;

namespace WcUiWpf.Helpers;

/// <summary>
/// 直接通过 Windows API 设置窗口透明度（不需要DLL注入）
/// 使用 EnumWindows 枚举所有顶层窗口，兼容多进程 Electron 应用
/// </summary>
public static class WindowTransparencyHelper
{
    // 支持的 AI APP 进程名
    private static readonly string[] SupportedApps = {
        "Doubao", "Code", "Gemini", "GeminiApp",
        "Qwen", "QwenDesktop", "qianwen", "Qianwen",
        "Codebuddy", "CodeBuddy", "CodeBuddy CN",
        "ollama", "Ollama", "ollama app", "Ollama App",
        "DeepSeek", "deepseek", "ChatGLM", "ZhipuAI",
        "Kimi", "kimi", "xiuai", "SparkDesk",
        "yuanbao", "Yuanbao",
    };

    // 排除列表（保护我们自己的窗口）
    private static readonly string[] ExcludedApps = { "WcUiWpf", "wc-core" };

    [DllImport("user32.dll")]
    private static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
    [DllImport("user32.dll")]
    private static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")]
    private static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport("user32.dll")]
    private static extern IntPtr GetParent(IntPtr hWnd);
    [DllImport("user32.dll")]
    private static extern int GetWindowLong(IntPtr hWnd, int nIndex);
    [DllImport("user32.dll")]
    private static extern int SetWindowLong(IntPtr hWnd, int nIndex, int dwNewLong);
    [DllImport("user32.dll")]
    private static extern bool SetLayeredWindowAttributes(IntPtr hWnd, uint crKey, byte bAlpha, uint dwFlags);

    private delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    private const int GWL_EXSTYLE = -20;
    private const int WS_EX_LAYERED = 0x80000;
    private const int LWA_ALPHA = 0x2;

    /// <summary>
    /// 对所有运行中的 AI APP 应用透明度
    /// </summary>
    public static int ApplyTransparency(byte alpha = 100)
    {
        int count = 0;
        foreach (var hWnd in FindAiWindows())
        {
            try
            {
                int exStyle = GetWindowLong(hWnd, GWL_EXSTYLE);
                SetWindowLong(hWnd, GWL_EXSTYLE, exStyle | WS_EX_LAYERED);
                SetLayeredWindowAttributes(hWnd, 0, alpha, LWA_ALPHA);
                count++;
            }
            catch { }
        }
        return count;
    }

    /// <summary>
    /// 恢复所有 AI APP 的透明度
    /// </summary>
    public static int RestoreTransparency()
    {
        int count = 0;
        foreach (var hWnd in FindAiWindows())
        {
            try
            {
                int exStyle = GetWindowLong(hWnd, GWL_EXSTYLE);
                SetWindowLong(hWnd, GWL_EXSTYLE, exStyle & ~WS_EX_LAYERED);
                count++;
            }
            catch { }
        }
        return count;
    }

    /// <summary>
    /// 通过 EnumWindows 查找所有 AI APP 的顶层窗口
    /// </summary>
    private static List<IntPtr> FindAiWindows()
    {
        var windows = new List<IntPtr>();
        var supportedPids = new HashSet<uint>();

        foreach (var appName in SupportedApps)
        {
            foreach (var p in Process.GetProcessesByName(appName))
            {
                if (ExcludedApps.Contains(p.ProcessName, StringComparer.OrdinalIgnoreCase))
                    continue;
                supportedPids.Add((uint)p.Id);
            }
        }

        EnumWindows((hWnd, lParam) =>
        {
            if (!IsWindowVisible(hWnd) || GetParent(hWnd) != IntPtr.Zero)
                return true;

            GetWindowThreadProcessId(hWnd, out uint pid);

            if (supportedPids.Contains(pid))
            {
                var title = new StringBuilder(256);
                GetWindowText(hWnd, title, 256);
                if (title.Length > 0)
                    windows.Add(hWnd);
            }
            return true;
        }, IntPtr.Zero);

        return windows;
    }
}