using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.IO;
using WcUiWpf.Helpers;

namespace WcUiWpf.Windows;

public partial class SettingsWindow : Window
{
    private bool _hasChanges;
    private AppConfig _originalConfig = null!;

    public SettingsWindow()
    {
        InitializeComponent();

        Loaded += (s, e) =>
        {
            _originalConfig = ConfigManager.Load();
            _hasChanges = false;
        };

        TitleBar.MouseLeftButtonDown += (s, e) => { if (e.ClickCount == 1) DragMove(); };

        BtnClose.Click += (s, e) => TryClose();

        BtnConfirm.Click += (s, e) =>
        {
            SaveConfig();
            _hasChanges = false;
            Close();
        };

        BtnCancel.Click += (s, e) => TryClose();

        BtnRestore.Click += (s, e) =>
        {
            _hasChanges = true;
        };

        // 清除日志缓存
        BtnClearLogs.Click += (s, e) =>
        {
            try
            {
                string appData = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "wallpaper-connecter");
                int cleared = 0;
                if (Directory.Exists(appData))
                {
                    foreach (var f in Directory.GetFiles(appData, "*.log"))
                    {
                        try { File.Delete(f); cleared++; } catch { }
                    }
                }
                MessageBox.Show($"已清除 {cleared} log files。", "Cache Cleared", MessageBoxButton.OK, MessageBoxImage.Information);
            }
            catch (Exception ex)
            {
                MessageBox.Show("清除失败: " + ex.Message, "Error", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        };

        ModalSave.Click += (s, e) =>
        {
            SaveConfig();
            UnsavedModal.Visibility = Visibility.Collapsed;
            Close();
        };
        ModalDontSave.Click += (s, e) =>
        {
            UnsavedModal.Visibility = Visibility.Collapsed;
            Close();
        };

        BtnConfirm.MouseEnter += (s, e) => BtnConfirm.Background = new SolidColorBrush(Color.FromArgb(0xFF, 0x1A, 0x8A, 0xE0));
        BtnConfirm.MouseLeave += (s, e) => BtnConfirm.Background = new SolidColorBrush(Color.FromArgb(0xFF, 0x00, 0x7A, 0xCC));
    }

    private void SaveConfig()
    {
        var cfg = ConfigManager.Load();
        ConfigManager.Save(cfg);
    }

    private void TryClose()
    {
        if (_hasChanges)
        {
            UnsavedModal.Visibility = Visibility.Visible;
        }
        else
        {
            Close();
        }
    }
}
