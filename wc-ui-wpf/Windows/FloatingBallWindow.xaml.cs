using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Animation;
using System.Windows.Threading;
using WcUiWpf.Helpers;

namespace WcUiWpf.Windows;

public partial class FloatingBallWindow : Window
{
    private const double HiddenTop = -52; // 只露底部4px
    private const double ShownTop = 0;

    private readonly DispatcherTimer _revealTimer;
    private readonly DispatcherTimer _hideTimer;
    private bool _isRevealed;
    private Point _dragStart;
    private bool _isDragging;
    private bool _suppressNextClick;

    // P/Invoke: 让窗口在所有虚拟桌面显示
    private const int GWL_EXSTYLE = -20;
    private const int WS_EX_TOOLWINDOW = 0x00000080;
    private const int WS_EX_NOACTIVATE = 0x08000000;
    [System.Runtime.InteropServices.DllImport("user32.dll")]
    private static extern int GetWindowLong(IntPtr hWnd, int nIndex);
    [System.Runtime.InteropServices.DllImport("user32.dll")]
    private static extern int SetWindowLong(IntPtr hWnd, int nIndex, int dwNewLong);

    public FloatingBallWindow()
    {
        InitializeComponent();
        AcrylicHelper.EnableAcrylic(this);

        Loaded += (s, e) =>
        {
            Left = SystemParameters.PrimaryScreenWidth / 2 - 28;
            Top = HiddenTop;
            App.Log("FloatingBallWindow loaded at hidden position");
            // 工具窗口样式 → 在所有虚拟桌面显示
            var helper = new System.Windows.Interop.WindowInteropHelper(this);
            var hwnd = helper.Handle;
            int exStyle = GetWindowLong(hwnd, GWL_EXSTYLE);
            SetWindowLong(hwnd, GWL_EXSTYLE, exStyle | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE);
        };

        // 1.5s 延迟显示
        _revealTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(1500) };
        _revealTimer.Tick += (s, e) => { _revealTimer.Stop(); ShowBall(); };

        // 2s 后自动隐藏
        _hideTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(2000) };
        _hideTimer.Tick += (s, e) => { _hideTimer.Stop(); HideBall(); };

        // 鼠标进入（即使只有4px可见也能触发）
        MouseEnter += (s, e) =>
        {
            if (!_isRevealed)
                _revealTimer.Start();
            _hideTimer.Stop();
        };
        MouseLeave += (s, e) =>
        {
            _revealTimer.Stop();
            if (_isRevealed)
                _hideTimer.Start();
        };

        // 点击/拖动
        BallCircle.MouseLeftButtonDown += OnBallDown;
        BallCircle.MouseLeftButtonUp += OnBallUp;
        BallCircle.MouseMove += OnBallMouseMove;

        // 右键菜单
        BallCircle.ContextMenu = BuildContextMenu();
        BallCircle.MouseRightButtonUp += (s, e) =>
        {
            _suppressNextClick = true;
            BallCircle.ContextMenu.IsOpen = true;
            e.Handled = true;
        };

        BallCircle.MouseEnter += (s, e) =>
            BallCircle.Fill = new SolidColorBrush(Color.FromArgb(0x60, 0xFF, 0xFF, 0xFF));
        BallCircle.MouseLeave += (s, e) =>
            BallCircle.Fill = new SolidColorBrush(Color.FromArgb(0x40, 0xFF, 0xFF, 0xFF));
    }

    private void OnBallDown(object sender, MouseButtonEventArgs e)
    {
        _dragStart = e.GetPosition(this);
        _isDragging = false;
        BallCircle.CaptureMouse();
    }

    private void OnBallMouseMove(object sender, MouseEventArgs e)
    {
        if (e.LeftButton == MouseButtonState.Pressed && BallCircle.IsMouseCaptured)
        {
            var pos = e.GetPosition(this);
            if (Math.Abs(pos.X - _dragStart.X) > 3 || Math.Abs(pos.Y - _dragStart.Y) > 3)
            {
                _isDragging = true;
                try { DragMove(); } catch { }
            }
        }
    }

    private void OnBallUp(object sender, MouseButtonEventArgs e)
    {
        BallCircle.ReleaseMouseCapture();
        if (_suppressNextClick) { _suppressNextClick = false; return; }
        if (_isDragging) return;

        // 点击悬浮球 = 打开/关闭音乐窗口
        App.ToggleMusic();
    }

    private void OnInjectTransparency(object sender, RoutedEventArgs e)
    {
        int count = WindowTransparencyHelper.ApplyTransparency(140);
        App.Log($"ApplyTransparency: {count} window(s)");
    }

    private void OnRestoreOpacity(object sender, RoutedEventArgs e)
    {
        int count = WindowTransparencyHelper.RestoreTransparency();
        App.Log($"RestoreTransparency: {count} window(s)");
    }

    private void OnToggleMusic(object sender, RoutedEventArgs e)
    {
        App.ToggleMusic();
    }

    private void OnOpenSettings(object sender, RoutedEventArgs e)
    {
        App.ShowSettings();
    }


    private void ShowBall()
    {
        _isRevealed = true;
        var anim = new DoubleAnimation(ShownTop, TimeSpan.FromMilliseconds(250))
        {
            EasingFunction = new CubicEase { EasingMode = EasingMode.EaseOut }
        };
        BeginAnimation(TopProperty, anim);
        _hideTimer.Start();
    }

    private void HideBall()
    {
        _isRevealed = false;
        var anim = new DoubleAnimation(HiddenTop, TimeSpan.FromMilliseconds(250))
        {
            EasingFunction = new CubicEase { EasingMode = EasingMode.EaseIn }
        };
        BeginAnimation(TopProperty, anim);
    }

    private ContextMenu BuildContextMenu()
    {
        var menu = new ContextMenu();

        var injectItem = new MenuItem { Header = "开启透明效果" };
        injectItem.Click += OnInjectTransparency;

        var restoreItem = new MenuItem { Header = "恢复透明度" };
        restoreItem.Click += OnRestoreOpacity;

        menu.Items.Add(injectItem);
        menu.Items.Add(restoreItem);
        menu.Items.Add(new Separator());

        var musicItem = new MenuItem { Header = I18n.Get("ball.music") };
        musicItem.Click += (s, e) => App.ToggleMusic();

        var settingsItem = new MenuItem { Header = I18n.Get("ball.settings") };
        settingsItem.Click += (s, e) => App.ShowSettings();

        menu.Items.Add(musicItem);
        menu.Items.Add(settingsItem);
        menu.Items.Add(new Separator());

        var exitItem = new MenuItem { Header = I18n.Get("ball.exit") };
        exitItem.Click += (s, e) =>
        {
            App.Core?.Stop();
            Application.Current.Shutdown();
        };

        menu.Items.Add(exitItem);

        return menu;
    }
}
