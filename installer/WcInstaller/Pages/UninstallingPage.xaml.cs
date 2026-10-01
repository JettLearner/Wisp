using System;
using System.Windows;
using System.Windows.Controls;
namespace WcInstaller.Pages
{
    public partial class UninstallingPage : Page
    {
        public UninstallingPage()
        {
            InitializeComponent();
            Loaded += async (s, e) =>
            {
                var path = InstallLogic.FindExistingInstall() ?? @"C:\Program Files\WallpaperConnecter";
                var progress = new Progress<int>(v => Dispatcher.Invoke(() => Progress.Value = v));
                var status = new Progress<string>(t => Dispatcher.Invoke(() => StatusText.Text = t));
                try
                {
                    await System.Threading.Tasks.Task.Run(() => InstallLogic.Uninstall(path, progress, status));
                    await System.Threading.Tasks.Task.Delay(500);
                    var win = (MainWindow)Window.GetWindow(this);
                    win.GoNext();
                }
                catch (Exception ex)
                {
                    MessageBox.Show("卸载失败: " + ex.Message, "Error", MessageBoxButton.OK, MessageBoxImage.Error);
                }
            };
        }
    }
}