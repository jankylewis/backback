namespace Harness.McpServer;

using System.ComponentModel;
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
}
