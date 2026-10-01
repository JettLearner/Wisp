namespace WcInstaller
{
    public class InstallState
    {
        public string InstallPath { get; set; } = @"C:\Program Files\WallpaperConnecter";
        public bool DesktopShortcut { get; set; } = true;
        public bool StartMenuShortcut { get; set; } = true;
        public bool LicenseAccepted { get; set; } = false;
        public bool LaunchAfterInstall { get; set; } = true;
    }
}