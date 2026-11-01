namespace Harness.NUnitApex;

using NUnit.Framework;

/// <summary>One spec per TRX outcome, for the test engine's end-to-end runs.</summary>
[TestFixture]
public sealed class OutcomeSpec
{
    [Test]
    [Category("Smoke")]
    public void Passes() => Assert.That(1 + 1, Is.EqualTo(2));

    [Test]
    public void Fails() => Assert.That(1 + 1, Is.EqualTo(3), "two is not three");

    [Test]
    [Ignore("skipped on purpose")]
    public void Skipped() => Assert.Pass();

    [Test]
    public void Undecided() => Assert.Inconclusive("not decided");
}
