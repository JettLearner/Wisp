using System.Windows;
using System.Windows.Controls;
namespace WcInstaller.Pages
{
    public partial class UninstallConfirmPage : Page
    {
        public UninstallConfirmPage()
        {
            InitializeComponent();
            Loaded += (s, e) =>
            {
                var path = InstallLogic.FindExistingInstall();
                PathText.Text = path != null ? "安装位置：" + path : "未找到安装位置，将使用默认路径卸载。";
            };
        }
    }
}