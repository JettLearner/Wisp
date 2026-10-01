using System.Windows;
using System.Windows.Controls;
namespace WcInstaller.Pages
{
    public partial class ShortcutPage : Page
    {
        public ShortcutPage()
        {
            InitializeComponent();
            Loaded += (s, e) =>
            {
                var win = (MainWindow)Window.GetWindow(this);
                DesktopCheck.IsChecked = win.State.DesktopShortcut;
                StartMenuCheck.IsChecked = win.State.StartMenuShortcut;
                LaunchCheck.IsChecked = win.State.LaunchAfterInstall;
            };
            DesktopCheck.Checked += (s, e) => { if (Window.GetWindow(this) is MainWindow w) w.State.DesktopShortcut = true; };
            DesktopCheck.Unchecked += (s, e) => { if (Window.GetWindow(this) is MainWindow w) w.State.DesktopShortcut = false; };
            StartMenuCheck.Checked += (s, e) => { if (Window.GetWindow(this) is MainWindow w) w.State.StartMenuShortcut = true; };
            StartMenuCheck.Unchecked += (s, e) => { if (Window.GetWindow(this) is MainWindow w) w.State.StartMenuShortcut = false; };
            LaunchCheck.Checked += (s, e) => { if (Window.GetWindow(this) is MainWindow w) w.State.LaunchAfterInstall = true; };
            LaunchCheck.Unchecked += (s, e) => { if (Window.GetWindow(this) is MainWindow w) w.State.LaunchAfterInstall = false; };
        }
    }
}