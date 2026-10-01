using System.Windows;
namespace WcInstaller
{
    public partial class App : Application
    {
        public static bool IsUninstall { get; private set; }
        protected override void OnStartup(StartupEventArgs e)
        {
            foreach (var arg in e.Args)
            {
                if (arg.Equals("/uninstall", System.StringComparison.OrdinalIgnoreCase))
                    IsUninstall = true;
            }

            // 检测 .NET 8 Desktop Runtime
            if (!DotNetChecker.IsNet8DesktopInstalled())
            {
                var result = MessageBox.Show(
                    "本程序需要 .NET 8 Desktop Runtime 才能运行。\n\n您的电脑尚未安装 .NET 8，是否现在前往下载？\n\n下载地址：" + DotNetChecker.DownloadUrl,
                    "需要 .NET 8",
                    MessageBoxButton.YesNo,
                    MessageBoxImage.Warning);
                if (result == MessageBoxResult.Yes)
                {
                    try { System.Diagnostics.Process.Start(DotNetChecker.DownloadUrl); } catch { }
                }
                Shutdown();
                return;
            }

            base.OnStartup(e);
        }
    }
}