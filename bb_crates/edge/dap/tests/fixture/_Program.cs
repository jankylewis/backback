using System;

class Program
{
    static int Main()
    {
        int count = 0;
        count += 1;
        count = count + 1;
        Console.WriteLine(count);
        try
        {
            throw new InvalidOperationException();
        }
        catch (InvalidOperationException)
        {
            Console.WriteLine(++count);
        }
        finally
        {
            Console.WriteLine(++count);
        }
        Console.WriteLine(count);
        return 0;
    }

    static void OtherMethod()
    {
        Console.WriteLine("other");
    }
}
