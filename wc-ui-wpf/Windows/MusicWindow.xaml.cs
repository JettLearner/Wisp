using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Windows;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Threading;
using Windows.Media.Control;
using Windows.Storage.Streams;
using WcUiWpf.Helpers;

namespace WcUiWpf.Windows;

public class LyricLine : INotifyPropertyChanged
{
    private string _text = "";
    private bool _isActive;
    private Brush _foreground = new SolidColorBrush(Color.FromArgb(0x50, 0xFF, 0xFF, 0xFF));
    private Brush _activeForeground = new SolidColorBrush(Color.FromArgb(0xF0, 0xFF, 0xFF, 0xFF));
    public string Text { get => _text; set { _text = value; OnPropertyChanged(nameof(Text)); } }
    public bool IsActive { get => _isActive; set { _isActive = value; OnPropertyChanged(nameof(IsActive)); } }
    public Brush Foreground { get => _foreground; set { _foreground = value; OnPropertyChanged(nameof(Foreground)); } }
    public Brush ActiveForeground { get => _activeForeground; set { _activeForeground = value; OnPropertyChanged(nameof(ActiveForeground)); } }
    public event PropertyChangedEventHandler? PropertyChanged;
    protected void OnPropertyChanged(string name) => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
}

public struct LyricEntry
{
    public TimeSpan Time;
    public string Text;
}

public partial class MusicWindow : Window
{
    private GlobalSystemMediaTransportControlsSessionManager? _smtcManager;
    private GlobalSystemMediaTransportControlsSession? _currentSession;
    private readonly DispatcherTimer _progressTimer;
    private static readonly HttpClient _http = new();
    private List<LyricEntry> _lyrics = new();
    private string _lastSongKey = "";
    private int _activeLyricIndex = -1;
    private double _currentSeconds;
    private double _totalSeconds = 215;

    static MusicWindow()
    {
        _http.DefaultRequestHeaders.Add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36");
        _http.DefaultRequestHeaders.Add("Referer", "https://music.163.com/");
    }

    public MusicWindow()
    {
        InitializeComponent();
        AcrylicHelper.EnableAcrylic(this, System.Windows.Media.Color.FromArgb(0, 0, 0, 0), 0);
        LyricItems.ItemsSource = LyricLines;
        SetLyricPlaceholder("等待播放音乐...");

        _progressTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(300) };
        _progressTimer.Tick += (s, e) => UpdatePlaybackState();

        Loaded += async (s, e) =>
        {
            var cfg = ConfigManager.Load();
            Left = cfg.MusicWindowX >= 0 ? cfg.MusicWindowX : SystemParameters.PrimaryScreenWidth - 340;
            Top = cfg.MusicWindowY >= 0 ? cfg.MusicWindowY : SystemParameters.PrimaryScreenHeight - 520;
            await InitSmtc();
            UpdateTextColor();
        };

        LocationChanged += (s, e) => UpdateTextColor();
        MusicGrid.MouseLeftButtonDown += (s, e) => { if (e.ClickCount == 1 && !IsOverButton(e)) DragMove(); };
        BtnClose.Click += (s, e) => { SavePosition(); Hide(); };
        BtnPlay.Click += async (s, e) => await TogglePlay();
        BtnPrev.Click += async (s, e) => await SkipPrevious();
        BtnNext.Click += async (s, e) => await SkipNext();

        BtnPlay.MouseEnter += (s, e) => BtnPlay.Background = new SolidColorBrush(Color.FromArgb(0x55, 0x80, 0xC0, 0xFF));
        BtnPlay.MouseLeave += (s, e) => BtnPlay.Background = new SolidColorBrush(Color.FromArgb(0x35, 0x80, 0xC0, 0xFF));
        foreach (var btn in new[] { BtnPrev, BtnNext, BtnClose })
        {
            btn.MouseEnter += (s, e) => btn.Foreground = new SolidColorBrush(Colors.White);
            btn.MouseLeave += (s, e) => btn.Foreground = new SolidColorBrush(Color.FromArgb(0xB0, 0xFF, 0xFF, 0xFF));
        }
    }

    public ObservableCollection<LyricLine> LyricLines { get; } = new();

    private async Task InitSmtc()
    {
        try
        {
            _smtcManager = await GlobalSystemMediaTransportControlsSessionManager.RequestAsync();
            _smtcManager.CurrentSessionChanged += OnCurrentSessionChanged;
            var session = _smtcManager.GetCurrentSession();
            if (session != null) await AttachSession(session);
            else SetNoMediaState();
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"SMTC init: {ex.Message}");
            SetNoMediaState();
        }
    }

    private async void OnCurrentSessionChanged(GlobalSystemMediaTransportControlsSessionManager sender, CurrentSessionChangedEventArgs args)
    {
        await Dispatcher.Invoke(async () =>
        {
            var session = sender.GetCurrentSession();
            if (session != null) await AttachSession(session);
            else SetNoMediaState();
        });
    }

    private async Task AttachSession(GlobalSystemMediaTransportControlsSession session)
    {
        if (_currentSession != null)
        {
            _currentSession.MediaPropertiesChanged -= OnMediaPropertiesChanged;
            _currentSession.PlaybackInfoChanged -= OnPlaybackInfoChanged;
        }
        _currentSession = session;
        session.MediaPropertiesChanged += OnMediaPropertiesChanged;
        session.PlaybackInfoChanged += OnPlaybackInfoChanged;
        await UpdateMediaProperties();
        UpdatePlaybackState();
    }

    private async void OnMediaPropertiesChanged(GlobalSystemMediaTransportControlsSession sender, MediaPropertiesChangedEventArgs args)
    {
        await Dispatcher.Invoke(async () => await UpdateMediaProperties());
    }

    private void OnPlaybackInfoChanged(GlobalSystemMediaTransportControlsSession sender, PlaybackInfoChangedEventArgs args)
    {
        Dispatcher.Invoke(UpdatePlaybackState);
    }

    private async Task UpdateMediaProperties()
    {
        if (_currentSession == null) return;
        try
        {
            var props = await _currentSession.TryGetMediaPropertiesAsync();
            if (props == null) return;

            var title = string.IsNullOrEmpty(props.Title) ? "未知歌曲" : props.Title;
            var artist = props.Artist;
            if (string.IsNullOrEmpty(artist) && !string.IsNullOrEmpty(props.AlbumArtist))
                artist = props.AlbumArtist;
            if (string.IsNullOrEmpty(artist)) artist = "—";

            TrackTitle.Text = title;
            TrackArtist.Text = artist;

            if (props.Thumbnail != null)
            {
                try
                {
                    using IRandomAccessStreamWithContentType stream = await props.Thumbnail.OpenReadAsync();
                    var bitmap = new BitmapImage();
                    bitmap.BeginInit();
                    bitmap.CacheOption = BitmapCacheOption.OnLoad;
                    bitmap.StreamSource = stream.AsStream();
                    bitmap.EndInit();
                    bitmap.Freeze();
                    CoverImage.Source = bitmap;
                    CoverImage.Visibility = Visibility.Visible;
                    CoverPlaceholder.Visibility = Visibility.Collapsed;
                }
                catch { }
            }

            var songKey = $"{title}|{artist}";
            if (songKey != _lastSongKey)
            {
                _lastSongKey = songKey;
                _ = SearchAndLoadLyrics(title, artist);
            }
        }
        catch { }
    }

    private async Task SearchAndLoadLyrics(string title, string artist)
    {
        SetLyricPlaceholder("正在搜索歌词...");
        try
        {
            var keyword = Uri.EscapeDataString($"{title} {artist}");
            var searchUrl = $"https://music.163.com/api/search/get/web?s={keyword}&type=1&limit=3";
            var searchResp = await _http.GetAsync(searchUrl);
            if (!searchResp.IsSuccessStatusCode) { SetLyricPlaceholder("歌词搜索失败"); return; }

            var searchJson = await searchResp.Content.ReadAsStringAsync();
            using var searchDoc = JsonDocument.Parse(searchJson);
            if (!searchDoc.RootElement.TryGetProperty("result", out var resultEl) ||
                !resultEl.TryGetProperty("songs", out var songs) ||
                songs.GetArrayLength() == 0)
            {
                SetLyricPlaceholder("暂无歌词");
                return;
            }

            var songId = songs[0].GetProperty("id").GetInt64();

            var lyricUrl = $"https://music.163.com/api/song/lyric?id={songId}&lv=1&kv=1&tv=-1";
            var lyricResp = await _http.GetAsync(lyricUrl);
            if (!lyricResp.IsSuccessStatusCode) { SetLyricPlaceholder("歌词获取失败"); return; }

            var lyricJson = await lyricResp.Content.ReadAsStringAsync();
            using var lyricDoc = JsonDocument.Parse(lyricJson);
            if (!lyricDoc.RootElement.TryGetProperty("lrc", out var lrcEl) ||
                !lrcEl.TryGetProperty("lyric", out var lyricStrEl))
            {
                SetLyricPlaceholder("纯音乐，请欣赏");
                return;
            }

            var lrcText = lyricStrEl.GetString();
            if (string.IsNullOrWhiteSpace(lrcText))
            {
                SetLyricPlaceholder("纯音乐，请欣赏");
                return;
            }

            _lyrics = ParseLrc(lrcText);
            if (_lyrics.Count == 0)
            {
                SetLyricPlaceholder("暂无歌词");
                return;
            }

            Dispatcher.Invoke(() =>
            {
                LyricLines.Clear();
                foreach (var entry in _lyrics)
                    LyricLines.Add(new LyricLine { Text = entry.Text });
                _activeLyricIndex = -1;
            });
        }
        catch (Exception ex)
        {
            SetLyricPlaceholder($"歌词加载失败");
        }
    }

    private static List<LyricEntry> ParseLrc(string lrc)
    {
        var entries = new List<LyricEntry>();
        var regex = new Regex(@"\[(\d{2}):(\d{2})(?:[.:](\d{1,3}))?\](.*)");
        foreach (var line in lrc.Split('\n'))
        {
            var trimmed = line.Trim();
            var matches = regex.Matches(trimmed);
            if (matches.Count == 0) continue;

            string text = "";
            var times = new List<TimeSpan>();
            foreach (Match m in matches)
            {
                int min = int.Parse(m.Groups[1].Value);
                int sec = int.Parse(m.Groups[2].Value);
                int ms = 0;
                if (m.Groups[3].Success)
                {
                    var msStr = m.Groups[3].Value;
                    if (msStr.Length == 1) msStr += "00";
                    else if (msStr.Length == 2) msStr += "0";
                    ms = int.Parse(msStr[..3]);
                }
                times.Add(TimeSpan.FromMinutes(min).Add(TimeSpan.FromSeconds(sec)).Add(TimeSpan.FromMilliseconds(ms)));
                text = m.Groups[4].Value.Trim();
            }
            if (string.IsNullOrEmpty(text)) continue;
            foreach (var t in times)
                entries.Add(new LyricEntry { Time = t, Text = text });
        }
        return entries.OrderBy(e => e.Time).ToList();
    }

    private void SetLyricPlaceholder(string text)
    {
        _lyrics.Clear();
        Dispatcher.Invoke(() =>
        {
            LyricLines.Clear();
            LyricLines.Add(new LyricLine { Text = text, IsActive = true });
            _activeLyricIndex = -1;
        });
    }

    private void UpdatePlaybackState()
    {
        if (_currentSession == null) { SetNoMediaState(); return; }
        try
        {
            var info = _currentSession.GetPlaybackInfo();
            bool isPlaying = info.PlaybackStatus == GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing;
            BtnPlay.Content = isPlaying ? "⏸" : "▶";
            if (isPlaying) _progressTimer.Start(); else _progressTimer.Stop();

            var timeline = _currentSession.GetTimelineProperties();
            if (timeline != null && timeline.EndTime.TotalSeconds > 0)
            {
                _totalSeconds = timeline.EndTime.TotalSeconds;
                _currentSeconds = timeline.Position.TotalSeconds;
                ProgressBar.Value = Math.Min(100, (_currentSeconds / _totalSeconds) * 100);
                TimeCurrent.Text = FormatTime(_currentSeconds);
                TimeTotal.Text = FormatTime(_totalSeconds);
                SyncLyric(_currentSeconds);
            }
        }
        catch { }
    }

    private void SyncLyric(double currentSeconds)
    {
        if (_lyrics.Count == 0) return;
        var currentTime = TimeSpan.FromSeconds(currentSeconds);

        int newIndex = -1;
        for (int i = 0; i < _lyrics.Count; i++)
        {
            if (_lyrics[i].Time <= currentTime)
                newIndex = i;
            else
                break;
        }

        if (newIndex != _activeLyricIndex && newIndex >= 0 && newIndex < LyricLines.Count)
        {
            if (_activeLyricIndex >= 0 && _activeLyricIndex < LyricLines.Count)
                LyricLines[_activeLyricIndex].IsActive = false;
            _activeLyricIndex = newIndex;
            LyricLines[_activeLyricIndex].IsActive = true;
            LyricScroll.ScrollToVerticalOffset(Math.Max(0, _activeLyricIndex * 24 - 48));
        }
    }

    private void SetNoMediaState()
    {
        TrackTitle.Text = "未在播放";
        TrackArtist.Text = "—";
        BtnPlay.Content = "▶";
        CoverImage.Visibility = Visibility.Collapsed;
        CoverPlaceholder.Visibility = Visibility.Visible;
        _progressTimer.Stop();
        SetLyricPlaceholder("播放音乐后自动同步歌词");
    }

    private async Task TogglePlay()
    {
        if (_currentSession == null) return;
        try
        {
            var info = _currentSession.GetPlaybackInfo();
            if (info.PlaybackStatus == GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing)
                await _currentSession.TryPauseAsync();
            else
                await _currentSession.TryPlayAsync();
        }
        catch { }
    }

    private async Task SkipPrevious()
    {
        if (_currentSession == null) return;
        try { await _currentSession.TrySkipPreviousAsync(); } catch { }
    }

    private async Task SkipNext()
    {
        if (_currentSession == null) return;
        try { await _currentSession.TrySkipNextAsync(); } catch { }
    }

    private static string FormatTime(double seconds)
    {
        int m = (int)(seconds / 60);
        int s = (int)(seconds % 60);
        return $"{m}:{s:D2}";
    }

    private bool IsOverButton(MouseButtonEventArgs e)
    {
        var source = e.OriginalSource as IInputElement;
        return source == BtnPlay || source == BtnPrev || source == BtnNext
            || source == BtnClose || source == ProgressBar;
    }

    private void SavePosition()
    {
        try
        {
            var cfg = ConfigManager.Load();
            cfg.MusicWindowX = (int)Left;
            cfg.MusicWindowY = (int)Top;
            ConfigManager.Save(cfg);
        }
        catch { }
    }


    /// <summary>
    /// 采样窗口中心位置的背景亮度（截屏一小块区域计算平均亮度）
    /// </summary>
    private double SampleBackgroundBrightness()
    {
        try
        {
            int cx = (int)(Left + Width / 2);
            int cy = (int)(Top + Height / 2);
            int sampleSize = 30;
            int x = cx - sampleSize / 2;
            int y = cy - sampleSize / 2;

            using var bmp = new System.Drawing.Bitmap(sampleSize, sampleSize);
            using var g = System.Drawing.Graphics.FromImage(bmp);
            g.CopyFromScreen(x, y, 0, 0, new System.Drawing.Size(sampleSize, sampleSize));

            long totalR = 0, totalG = 0, totalB = 0;
            int count = 0;
            for (int px = 0; px < sampleSize; px += 3)
            {
                for (int py = 0; py < sampleSize; py += 3)
                {
                    var c = bmp.GetPixel(px, py);
                    totalR += c.R;
                    totalG += c.G;
                    totalB += c.B;
                    count++;
                }
            }
            if (count == 0) return 128;
            double avgR = totalR / count;
            double avgG = totalG / count;
            double avgB = totalB / count;
            // 相对亮度公式：Y = 0.299R + 0.587G + 0.114B
            return 0.299 * avgR + 0.587 * avgG + 0.114 * avgB;
        }
        catch
        {
            return 128; // 默认中间值
        }
    }

    /// <summary>
    /// 根据背景亮度切换文字颜色（深色背景用白字，浅色背景用黑字）
    /// </summary>
    private void UpdateTextColor()
    {
        double brightness = SampleBackgroundBrightness();
        bool darkBg = brightness < 128;

        if (darkBg)
        {
            // 深色背景：白色文字
            HeaderLabel.Foreground = new SolidColorBrush(Color.FromArgb(0x60, 0xFF, 0xFF, 0xFF));
            BtnClose.Foreground = new SolidColorBrush(Color.FromArgb(0x60, 0xFF, 0xFF, 0xFF));
            CoverPlaceholder.Foreground = new SolidColorBrush(Color.FromArgb(0x30, 0xFF, 0xFF, 0xFF));
            TrackTitle.Foreground = new SolidColorBrush(Color.FromArgb(0xF0, 0xFF, 0xFF, 0xFF));
            TrackArtist.Foreground = new SolidColorBrush(Color.FromArgb(0x70, 0xFF, 0xFF, 0xFF));
            TimeCurrent.Foreground = new SolidColorBrush(Color.FromArgb(0x50, 0xFF, 0xFF, 0xFF));
            TimeTotal.Foreground = new SolidColorBrush(Color.FromArgb(0x50, 0xFF, 0xFF, 0xFF));
            BtnPrev.Foreground = new SolidColorBrush(Color.FromArgb(0xC0, 0xFF, 0xFF, 0xFF));
            BtnNext.Foreground = new SolidColorBrush(Color.FromArgb(0xC0, 0xFF, 0xFF, 0xFF));
            BtnPlay.Foreground = new SolidColorBrush(Colors.White);
            ProgressBar.Foreground = new SolidColorBrush(Color.FromArgb(0x80, 0xC0, 0xE0, 0xFF));
            ProgressBar.Background = new SolidColorBrush(Color.FromArgb(0x20, 0xFF, 0xFF, 0xFF));
            RootBorder.BorderBrush = new SolidColorBrush(Color.FromArgb(0x35, 0xFF, 0xFF, 0xFF));
            CoverBorder.BorderBrush = new SolidColorBrush(Color.FromArgb(0x25, 0xFF, 0xFF, 0xFF));
            CoverBorder.Background = new SolidColorBrush(Color.FromArgb(0x20, 0xFF, 0xFF, 0xFF));

            var lyricFg = new SolidColorBrush(Color.FromArgb(0x50, 0xFF, 0xFF, 0xFF));
            var lyricActiveFg = new SolidColorBrush(Color.FromArgb(0xF0, 0xFF, 0xFF, 0xFF));
            foreach (var line in LyricLines)
            {
                line.Foreground = lyricFg;
                line.ActiveForeground = lyricActiveFg;
            }
        }
        else
        {
            // 浅色背景：黑色文字
            HeaderLabel.Foreground = new SolidColorBrush(Color.FromArgb(0x60, 0x00, 0x00, 0x00));
            BtnClose.Foreground = new SolidColorBrush(Color.FromArgb(0x60, 0x00, 0x00, 0x00));
            CoverPlaceholder.Foreground = new SolidColorBrush(Color.FromArgb(0x30, 0x00, 0x00, 0x00));
            TrackTitle.Foreground = new SolidColorBrush(Color.FromArgb(0xF0, 0x00, 0x00, 0x00));
            TrackArtist.Foreground = new SolidColorBrush(Color.FromArgb(0x70, 0x00, 0x00, 0x00));
            TimeCurrent.Foreground = new SolidColorBrush(Color.FromArgb(0x50, 0x00, 0x00, 0x00));
            TimeTotal.Foreground = new SolidColorBrush(Color.FromArgb(0x50, 0x00, 0x00, 0x00));
            BtnPrev.Foreground = new SolidColorBrush(Color.FromArgb(0xC0, 0x00, 0x00, 0x00));
            BtnNext.Foreground = new SolidColorBrush(Color.FromArgb(0xC0, 0x00, 0x00, 0x00));
            BtnPlay.Foreground = new SolidColorBrush(Colors.Black);
            ProgressBar.Foreground = new SolidColorBrush(Color.FromArgb(0x80, 0x40, 0x60, 0x80));
            ProgressBar.Background = new SolidColorBrush(Color.FromArgb(0x20, 0x00, 0x00, 0x00));
            RootBorder.BorderBrush = new SolidColorBrush(Color.FromArgb(0x35, 0x00, 0x00, 0x00));
            CoverBorder.BorderBrush = new SolidColorBrush(Color.FromArgb(0x25, 0x00, 0x00, 0x00));
            CoverBorder.Background = new SolidColorBrush(Color.FromArgb(0x20, 0x00, 0x00, 0x00));

            var lyricFg = new SolidColorBrush(Color.FromArgb(0x50, 0x00, 0x00, 0x00));
            var lyricActiveFg = new SolidColorBrush(Color.FromArgb(0xF0, 0x00, 0x00, 0x00));
            foreach (var line in LyricLines)
            {
                line.Foreground = lyricFg;
                line.ActiveForeground = lyricActiveFg;
            }
        }
    }

    protected override void OnClosing(CancelEventArgs e)
    {
        SavePosition();
        base.OnClosing(e);
    }
}
