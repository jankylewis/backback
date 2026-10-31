namespace Harness.Core;

public sealed class Greeter
{
    public string Hello(string name)
    {
        string who = string.IsNullOrWhiteSpace(name) ? "there" : name;
        return $"hi {who}";
    }
}
