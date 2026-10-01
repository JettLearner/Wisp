using Microsoft.Win32;
using System;
using System.IO;

namespace WcInstaller
{
    public static class DotNetChecker
    {
        public static bool IsNet8DesktopInstalled()
        {
            // 方法1：检查注册表
            try
            {
                using var key = Registry.LocalMachine.OpenSubKey(@"SOFTWARE\dotnet\Setup\InstalledVersions\x64\sharedfx\Microsoft.WindowsDesktop.App");
                if (key != null)
                {
                    foreach (var name in key.GetValueNames())
                    {
                        if (name.StartsWith("8.")) return true;
                    }
                }
            }
            catch { }

            // 方法2：检查目录
            try
            {
                string sharedDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), "dotnet", "shared", "Microsoft.WindowsDesktop.App");
                if (Directory.Exists(sharedDir))
                {
                    foreach (var dir in Directory.GetDirectories(sharedDir))
                    {
                        string ver = Path.GetFileName(dir);
                        if (ver.StartsWith("8.")) return true;
                    }
                }
            }
            catch { }

            return false;
        }

        public static readonly string DownloadUrl = "https://dotnet.microsoft.com/zh-cn/download/dotnet/8.0";
    }
}