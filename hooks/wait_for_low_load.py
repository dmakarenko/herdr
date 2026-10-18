#!/usr/bin/env python3
# ABOUTME: Holds the pre-push test run until the 1-minute load is below the CPU count, or fails after a limit.
import os
import sys
import time


def wait_for_low_load(read_load, cpus, max_seconds, interval, monotonic, sleep, report):
    start = monotonic()
    while True:
        load = read_load()
        if load < cpus:
            return True
        waited = monotonic() - start
        if waited >= max_seconds:
            report(f"load {load:.1f} still >= {cpus} after {waited:.0f} s; push again later")
            return False
        report(f"load {load:.1f} >= {cpus} CPUs; waiting {interval} s before the tests")
        sleep(interval)


def main():
    ok = wait_for_low_load(
        read_load=lambda: os.getloadavg()[0],
        cpus=os.cpu_count() or 1,
        max_seconds=900,
        interval=10,
        monotonic=time.monotonic,
        sleep=time.sleep,
        report=lambda message: print(message, flush=True),
    )
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
