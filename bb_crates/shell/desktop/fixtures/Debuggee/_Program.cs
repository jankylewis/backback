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
        Console.WriteLine(total);
        return 0;
    }

    static int Square(int n)
    {
        int result = n * n;
        return result;
    }
}
