using System.Windows;
using System.Windows.Controls;
namespace WcInstaller.Pages
{
    public partial class WelcomePage : Page
    {
        public WelcomePage()
        {
            InitializeComponent();
            Loaded += (s, e) =>
            {
                var existing = InstallLogic.FindExistingInstall();
                if (existing != null)
                {
                    bool integrity = true;
                    string msg = integrity
                        ? $"Wisp is already installed在：\n{existing}\n\nDo you want to overwrite?？"
                        : $"检测到 Wallpaper Connecter 安装目录（Files are incomplete）：\n{existing}\n\n是否重新安装？";
                    var result = MessageBox.Show(msg, "已检测到安装", MessageBoxButton.YesNo, MessageBoxImage.Question);
                    if (result == MessageBoxResult.No)
                    {
                        var win = (MainWindow)Window.GetWindow(this);
                        win.Close();
                    }
                }
            };
        }
    }
}