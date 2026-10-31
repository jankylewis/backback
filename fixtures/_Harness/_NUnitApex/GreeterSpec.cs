namespace Harness.NUnitApex;

using Harness.Core;
using NUnit.Framework;

[TestFixture]
public sealed class GreeterSpec
{
    [TestCase("Vinh", "hi Vinh")]
    [TestCase("", "hi there")]
    public void Greets(string name, string expected)
    {
        Greeter greeter = new Greeter();
        string actual = greeter.Hello(name);
        Assert.That(actual, Is.EqualTo(expected));
    }
}
