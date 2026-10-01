using System.Windows;
using System.Windows.Controls;
using WcInstaller.Pages;

namespace WcInstaller
{
    public partial class MainWindow : Window
    {
        public InstallState State { get; } = new InstallState();
        private int _currentIndex = 0;
        private readonly Page[] _installPages;
        private readonly Page[] _uninstallPages;

        public MainWindow()
        {
            InitializeComponent();

            _installPages = new Page[]
            {
                new WelcomePage(),
                new LicensePage(),
                new InstallPathPage(),
                new ShortcutPage(),
                new InstallingPage(),
                new FinishPage(),
            };

            _uninstallPages = new Page[]
            {
                new UninstallConfirmPage(),
                new UninstallingPage(),
                new UninstallFinishPage(),
            };

            var pages = App.IsUninstall ? _uninstallPages : _installPages;
            _currentIndex = 0;
            ContentFrame.Navigate(pages[0]);
            UpdateButtons();

            if (App.IsUninstall)
                Title = "Wallpaper Connecter 卸载向导";
        }

        private void UpdateButtons()
        {
            var pages = App.IsUninstall ? _uninstallPages : _installPages;
            BtnBack.Visibility = _currentIndex > 0 ? Visibility.Visible : Visibility.Collapsed;
            BtnNext.Content = _currentIndex == pages.Length - 1 ? "完成" : "下一步";
            BtnCancel.Visibility = _currentIndex < pages.Length - 1 ? Visibility.Visible : Visibility.Collapsed;

            // 许可协议页：必须接受才能下一步
            if (!App.IsUninstall && _currentIndex == 1 && !State.LicenseAccepted)
                BtnNext.IsEnabled = false;
            else
                BtnNext.IsEnabled = true;

            // 安装中/卸载中页面：禁用按钮
            if ((!App.IsUninstall && _currentIndex == 4) || (App.IsUninstall && _currentIndex == 1))
            {
                BtnBack.IsEnabled = false;
                BtnNext.IsEnabled = false;
                BtnCancel.IsEnabled = false;
            }
        }

        public void GoNext()
        {
            var pages = App.IsUninstall ? _uninstallPages : _installPages;
            if (_currentIndex < pages.Length - 1)
            {
                _currentIndex++;
                ContentFrame.Navigate(pages[_currentIndex]);
                UpdateButtons();
            }
            else
            {
                Close();
            }
        }

        private void BtnNext_Click(object sender, RoutedEventArgs e)
        {
            // 安装路径页：验证路径
            if (!App.IsUninstall && _currentIndex == 2)
            {
                var page = (InstallPathPage)ContentFrame.Content;
                if (!page.Validate()) return;
            }
            GoNext();
        }

        private void BtnBack_Click(object sender, RoutedEventArgs e)
        {
            var pages = App.IsUninstall ? _uninstallPages : _installPages;
            if (_currentIndex > 0)
            {
                _currentIndex--;
                ContentFrame.Navigate(pages[_currentIndex]);
                UpdateButtons();
            }
        }

        private void BtnCancel_Click(object sender, RoutedEventArgs e)
        {
            if (MessageBox.Show("确定要取消安装吗？", "确认", MessageBoxButton.YesNo, MessageBoxImage.Question) == MessageBoxResult.Yes)
                Close();
        }
    }
}