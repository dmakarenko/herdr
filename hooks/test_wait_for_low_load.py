# ABOUTME: Tests for the pre-push wait that holds the test run until this machine's load is low.
import unittest

from wait_for_low_load import wait_for_low_load


class FakeClock:
    def __init__(self):
        self.now = 0.0

    def monotonic(self):
        return self.now

    def sleep(self, seconds):
        self.now += seconds


def run(loads, cpus=18, max_seconds=900, interval=10):
    clock = FakeClock()
    readings = iter(loads)
    messages = []
    ok = wait_for_low_load(
        read_load=lambda: next(readings),
        cpus=cpus,
        max_seconds=max_seconds,
        interval=interval,
        monotonic=clock.monotonic,
        sleep=clock.sleep,
        report=messages.append,
    )
    return ok, clock.now, messages


class WaitForLowLoadTests(unittest.TestCase):
    def test_low_load_passes_without_waiting(self):
        ok, waited, _ = run([4.0])
        self.assertTrue(ok)
        self.assertEqual(waited, 0)

    def test_load_equal_to_cpu_count_is_too_high(self):
        ok, waited, _ = run([18.0, 17.9])
        self.assertTrue(ok)
        self.assertEqual(waited, 10)

    def test_waits_until_load_drops(self):
        ok, waited, messages = run([90.0, 40.0, 12.0])
        self.assertTrue(ok)
        self.assertEqual(waited, 20)
        self.assertIn("load 90.0 >= 18 CPUs", messages[0])

    def test_fails_when_load_stays_high_past_the_limit(self):
        ok, waited, messages = run([100.0] * 200)
        self.assertFalse(ok)
        self.assertEqual(waited, 900)
        self.assertIn("still >= 18 after 900 s", messages[-1])


if __name__ == "__main__":
    unittest.main()
