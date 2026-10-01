using System;
using System.Diagnostics;
using System.IO;
using System.IO.Compression;
using System.Reflection;
using System.Windows;

namespace WcInstaller
{
    public static class InstallLogic
    {
        public static void Install(InstallState state, IProgress<int> progress, IProgress<string> status)
        {
            // 1. 关闭正在运行的程序（执行两次，等够时间）
            status.Report("Closing running program......");
            Run("taskkill", "/F /IM WcUiWpf.exe");
            Run("taskkill", "/F /IM wc-core.exe");
            System.Threading.Thread.Sleep(3000);
            Run("taskkill", "/F /IM WcUiWpf.exe");
            Run("taskkill", "/F /IM wc-core.exe");
            System.Threading.Thread.Sleep(2000);
            progress.Report(10);

            // 2. 解压
            status.Report("Extracting files......");
            string tempDir = Path.Combine(Path.GetTempPath(), "WcInstall_" + Guid.NewGuid().ToString("N").Substring(0, 8));
            Directory.CreateDirectory(tempDir);
            try
            {
                using (Stream? payload = Assembly.GetExecutingAssembly().GetManifestResourceStream("WcInstaller.payload.zip"))
                {
                    if (payload != null)
                    {
                        using (var archive = new ZipArchive(payload, ZipArchiveMode.Read))
                        {
                            archive.ExtractToDirectory(tempDir, true);
                        }
                    }
                }
                progress.Report(30);

                // 3. 创建目录并复制
                status.Report("Copying files......");
                if (!Directory.Exists(state.InstallPath))
                    Directory.CreateDirectory(state.InstallPath);
                CopyDir(tempDir, state.InstallPath);
                progress.Report(70);

                // 4. 快捷方式
                if (state.DesktopShortcut)
                {
                    status.Report("创建桌面快捷方式...");
                    MakeShortcut(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Desktop), "Wallpaper Connecter.lnk"),
                        Path.Combine(state.InstallPath, "wc-core.exe"), state.InstallPath);
                }
                progress.Report(85);

                if (state.StartMenuShortcut)
                {
                    status.Report("创建开始菜单快捷方式...");
                    MakeShortcut(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Programs), "Wallpaper Connecter.lnk"),
                        Path.Combine(state.InstallPath, "wc-core.exe"), state.InstallPath);
                }
                progress.Report(95);

                // 5. 写卸载 bat
                string bat = "@echo off\r\nchcp 65001 >nul\r\ntitle Uninstall Wisp\r\necho Uninstalling......\r\ntaskkill /F /IM WcUiWpf.exe >nul 2>&1\r\ntaskkill /F /IM wc-core.exe >nul 2>&1\r\ntimeout /t 2 /nobreak >nul\r\nrmdir /S /Q \"" + state.InstallPath + "\"\r\ndel /Q \"%USERPROFILE%\\Desktop\\Wallpaper Connecter.lnk\" >nul 2>&1\r\ndel /Q \"%APPDATA%\\Microsoft\\Windows\\Start Menu\\Programs\\Wallpaper Connecter.lnk\" >nul 2>&1\r\necho Uninstall Complete！\r\npause\r\n";
                File.WriteAllText(Path.Combine(state.InstallPath, "uninstall.bat"), bat, new System.Text.UTF8Encoding(true));
                progress.Report(100);
                status.Report("Installation Complete！");
            }
            finally
            {
                try { Directory.Delete(tempDir, true); } catch { }
            }
        }

        public static void Uninstall(string installPath, IProgress<int> progress, IProgress<string> status)
        {
            status.Report("Closing program......");
            Run("taskkill", "/F /IM WcUiWpf.exe");
            Run("taskkill", "/F /IM wc-core.exe");
            System.Threading.Thread.Sleep(2000);
            progress.Report(20);

            status.Report("Removing files......");
            if (Directory.Exists(installPath))
                Directory.Delete(installPath, true);
            progress.Report(70);

            status.Report("Removing shortcuts......");
            try { File.Delete(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Desktop), "Wallpaper Connecter.lnk")); } catch { }
            try { File.Delete(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Programs), "Wallpaper Connecter.lnk")); } catch { }
            progress.Report(100);
            status.Report("Uninstall Complete！");
        }

        public static string? FindExistingInstall()
        {
            string[] candidates = { @"C:\Program Files\WallpaperConnecter", @"C:\Program Files (x86)\WallpaperConnecter" };
            foreach (var p in candidates)
                if (File.Exists(Path.Combine(p, "wc-core.exe"))) return p;
            return null;
        }

        private static void Run(string exe, string args)
        {
            try { Process.Start(new ProcessStartInfo(exe, args) { CreateNoWindow = true, UseShellExecute = false })?.WaitForExit(5000); } catch { }
        }

        private static void CopyDir(string src, string dst)
        {
            foreach (string f in Directory.GetFiles(src))
                File.Copy(f, Path.Combine(dst, Path.GetFileName(f)), true);
            foreach (string d in Directory.GetDirectories(src))
            {
                string nd = Path.Combine(dst, Path.GetFileName(d));
                Directory.CreateDirectory(nd);
                CopyDir(d, nd);
            }
        }

        private static void MakeShortcut(string lnk, string target, string workDir)
        {
            dynamic shell = Activator.CreateInstance(Type.GetTypeFromProgID("WScript.Shell")!)!;
            dynamic sc = shell.CreateShortcut(lnk);
            sc.TargetPath = target;
            sc.WorkingDirectory = workDir;
            sc.IconLocation = target + ",0";
            sc.Save();
        }
    }
}