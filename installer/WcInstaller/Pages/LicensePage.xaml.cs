using System.Windows;
using System.Windows.Controls;
namespace WcInstaller.Pages
{
    public partial class LicensePage : Page
    {
        public LicensePage() { InitializeComponent(); }
        private void AcceptCheck_Checked(object sender, RoutedEventArgs e)
        {
            var win = (MainWindow)Window.GetWindow(this);
            win.State.LicenseAccepted = true;
            win.Dispatcher.Invoke(() => { var f = typeof(MainWindow).GetMethod("UpdateButtons", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance); f?.Invoke(win, null); });
        }
        private void AcceptCheck_Unchecked(object sender, RoutedEventArgs e)
        {
            var win = (MainWindow)Window.GetWindow(this);
            win.State.LicenseAccepted = false;
            win.Dispatcher.Invoke(() => { var f = typeof(MainWindow).GetMethod("UpdateButtons", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance); f?.Invoke(win, null); });
        }
    }
}