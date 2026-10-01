using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media;

namespace WcUiWpf.Helpers;

/// <summary>
/// 通过 Windows 原生 SetWindowCompositionAttribute API 实现 Acrylic 毛玻璃效果。
/// 这是 Windows 10 1803+ 的原生 API，比 CSS backdrop-filter 可靠得多。
/// </summary>
public static class AcrylicHelper
{
    [DllImport("user32.dll")]
    private static extern int SetWindowCompositionAttribute(IntPtr hwnd, ref WindowCompositionAttributeData data);

    private enum AccentState
    {
        ACCENT_DISABLED = 0,
        ACCENT_ENABLE_GRADIENT = 1,
        ACCENT_ENABLE_TRANSPARENTGRADIENT = 2,
        ACCENT_ENABLE_BLURBEHIND = 3,
        ACCENT_ENABLE_ACRYLICBLURBEHIND = 4,
        ACCENT_INVALID_STATE = 5
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct AccentPolicy
    {
        public AccentState AccentState;
        public int AccentFlags;
        public int GradientColor; // ABGR format
        public int AnimationId;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct WindowCompositionAttributeData
    {
        public WindowCompositionAttribute Attribute;
        public IntPtr Data;
        public int SizeOfData;
    }

    private enum WindowCompositionAttribute
    {
        WCA_ACCENT_POLICY = 19
    }

    /// <summary>
    /// 给窗口应用 Acrylic 毛玻璃效果。
    /// </summary>
    /// <param name="window">目标窗口</param>
    /// <param name="tintColor">色调颜色（半透明叠加在模糊上）</param>
    /// <param name="tintOpacity">色调不透明度 0.0~1.0</param>
    private static Color _tintColor = Color.FromArgb(0x4D, 0x33, 0x33, 0x33);

    public static void EnableAcrylic(Window window, Color? tintColor = null, double tintOpacity = 0.3)
    {
        _tintColor = tintColor ?? Color.FromArgb((byte)(tintOpacity * 255), 0x33, 0x33, 0x33);
        if (window.IsLoaded)
            Apply(window);
        else
            window.Loaded += (s, e) => Apply(window);
    }

    private static void Apply(Window window)
    {
        try
        {
            var helper = new WindowInteropHelper(window);
            IntPtr hwnd = helper.Handle;
            if (hwnd == IntPtr.Zero) return;

            var accent = new AccentPolicy
            {
                AccentState = AccentState.ACCENT_ENABLE_ACRYLICBLURBEHIND,
                AccentFlags = 2,
                GradientColor = ToAbgr(_tintColor),
                AnimationId = 0
            };

            int accentSize = Marshal.SizeOf(accent);
            IntPtr accentPtr = Marshal.AllocHGlobal(accentSize);
            Marshal.StructureToPtr(accent, accentPtr, false);

            var data = new WindowCompositionAttributeData
            {
                Attribute = WindowCompositionAttribute.WCA_ACCENT_POLICY,
                Data = accentPtr,
                SizeOfData = accentSize
            };

            SetWindowCompositionAttribute(hwnd, ref data);
            Marshal.FreeHGlobal(accentPtr);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Acrylic apply failed: {ex.Message}");
        }
    }

    /// <summary>
    /// 禁用 Acrylic，恢复普通窗口。
    /// </summary>
    public static void DisableAcrylic(Window window)
    {
        var helper = new WindowInteropHelper(window);
        IntPtr hwnd = helper.Handle;
        if (hwnd == IntPtr.Zero) return;

        var accent = new AccentPolicy { AccentState = AccentState.ACCENT_DISABLED };
        int size = Marshal.SizeOf(accent);
        IntPtr ptr = Marshal.AllocHGlobal(size);
        Marshal.StructureToPtr(accent, ptr, false);

        var data = new WindowCompositionAttributeData
        {
            Attribute = WindowCompositionAttribute.WCA_ACCENT_POLICY,
            Data = ptr,
            SizeOfData = size
        };

        SetWindowCompositionAttribute(hwnd, ref data);
        Marshal.FreeHGlobal(ptr);
    }

    private static int ToAbgr(Color c)
    {
        // ABGR: 0xAABBGGRR
        return (c.A << 24) | (c.B << 16) | (c.G << 8) | c.R;
    }
}
