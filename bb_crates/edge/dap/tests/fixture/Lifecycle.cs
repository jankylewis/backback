using System;
using System.Threading;

static class Lifecycle
{
    public static int Run(string mode)
    {
        switch (mode)
        {
            case "exit3":
                return 3;
            case "spin":
                return Spin();
            case "await":
                return AwaitRelease();
            default:
                return 64;
        }
    }

    static int Spin()
    {
        int tick = 0;
        while (tick < 600)
        {
            tick = Advance(tick);
            Thread.Sleep(100);
        }
        return 1;
    }

    static int Advance(int tick)
    {
        int next = tick + 1;
        return next;
    }

    static int AwaitRelease()
    {
        Console.WriteLine("ready");
        Console.ReadLine();
        return 5;
    }
}
