using System;
using System.Windows;
using System.Windows.Controls;
namespace WcInstaller.Pages
{
    public partial class InstallingPage : Page
    {
        public InstallingPage()
        {
            InitializeComponent();
            Loaded += async (s, e) =>
            {
                var win = (MainWindow)Window.GetWindow(this);
                var progress = new Progress<int>(v => Dispatcher.Invoke(() => Progress.Value = v));
                var status = new Progress<string>(t => Dispatcher.Invoke(() => StatusText.Text = t));
                try
                {
                    await System.Threading.Tasks.Task.Run(() => InstallLogic.Install(win.State, progress, status));
                    await System.Threading.Tasks.Task.Delay(500);
                    win.GoNext();
                    if (win.State.LaunchAfterInstall)
                    {
                        try { System.Diagnostics.Process.Start(System.IO.Path.Combine(win.State.InstallPath, "wc-core.exe")); } catch { }
                    }
                }
                catch (Exception ex)
                {
                    MessageBox.Show("安装失败: " + ex.Message + "\n\n程序将关闭。", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                    win.Close();
                    return;
                }
            };
        }
    }
}