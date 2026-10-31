namespace Harness.XunitPlaywright;

using System.Threading.Tasks;
using Microsoft.Playwright;
using Xunit;

public sealed class HomePageSpec
{
    [Fact]
    public async Task TitleIsShown()
    {
        IPlaywright playwright = await Playwright.CreateAsync();
        IBrowser browser = await playwright.Chromium.LaunchAsync();
        IPage page = await browser.NewPageAsync();
        await page.SetContentAsync("<title>backback</title>");
        string title = await page.TitleAsync();
        Assert.Equal("backback", title);
    }
}
