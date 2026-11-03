using System;

class Program
{
    static int Main()
    {
        int total = 0;
        for (int i = 1; i <= 3; i++)
        {
            total += Square(i);
        }
        Console.WriteLine(Report(total));
        return 0;
    }

    static int Square(int n)
    {
        int result = n * n;
        return result;
    }

    static int Report(int total)
    {
        try
        {
            Fail(total);
            return -1;
        }
        catch (InvalidOperationException)
        {
            return total;
        }
    }

    static void Fail(int total)
    {
        throw new InvalidOperationException($"total was {total}");
    }
}
