namespace Harness.McpServer;

using System.ComponentModel;
using System.Threading.Tasks;
using Harness.Core;
using ModelContextProtocol.Server;

[McpServerToolType]
public static class GreetTool
{
    [McpServerTool, Description("Greets a person by name.")]
    public static string Greet(string name)
    {
        Greeter greeter = new Greeter();
        return greeter.Hello(name);
    }

    [McpServerTool, Description("Waits so the client can test a server crash.")]
    public static async Task<string> Hold()
    {
        await Task.Delay(10000);
        return "held";
    }
}
