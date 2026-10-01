using System.Windows;
using System.Windows.Controls;
using Microsoft.Win32;
namespace WcInstaller.Pages
{
    public partial class InstallPathPage : Page
    {
        public InstallPathPage()
        {
            InitializeComponent();
            Loaded += (s, e) => { var win = (MainWindow)Window.GetWindow(this); PathBox.Text = win.State.InstallPath; };
        }
        private void Browse_Click(object sender, RoutedEventArgs e)
        {
            var dialog = new OpenFolderDialog { Title = "选择安装目录" };
            if (dialog.ShowDialog() == true) PathBox.Text = dialog.FolderName;
        }
        public bool Validate()
        {
            var path = PathBox.Text.Trim();
            if (string.IsNullOrEmpty(path)) { ErrorText.Text = "请输入安装路径"; ErrorText.Visibility = Visibility.Visible; return false; }
            try { System.IO.Path.GetFullPath(path); } catch { ErrorText.Text = "路径格式不正确"; ErrorText.Visibility = Visibility.Visible; return false; }
            var win = (MainWindow)Window.GetWindow(this);
            win.State.InstallPath = path;
            ErrorText.Visibility = Visibility.Collapsed;
            return true;
        }
    }
}