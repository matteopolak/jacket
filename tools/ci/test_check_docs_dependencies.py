"""The npm docs builder obeys the same age gate as the compiler dependency graph."""

import base64
import json
from copy import deepcopy
from datetime import datetime, timedelta, timezone
import unittest
from unittest.mock import patch

import check_docs_dependencies as policy

NOW = datetime(2026, 11, 4, tzinfo=timezone.utc)
INTEGRITY = "sha512-" + base64.b64encode(bytes(64)).decode()


def fixture():
    package = {
        "name": "owned-builder",
        "version": "fixture",
        "packageManager": "npm@11.6.2",
        "dependencies": {"owned": "1.2.3"},
    }
    lock = {
        "name": package["name"],
        "version": package["version"],
        "lockfileVersion": 3,
        "packages": {
            "": {key: package[key] for key in ("name", "version", "dependencies")},
            "node_modules/owned": {
                "version": "1.2.3",
                "resolved": "https://registry.npmjs.org/owned/-/owned-1.2.3.tgz",
                "integrity": INTEGRITY,
            },
            "node_modules/owned/node_modules/transitive": {
                "version": "2.3.4",
                "resolved": "https://registry.npmjs.org/transitive/-/transitive-2.3.4.tgz",
                "integrity": INTEGRITY,
            },
        },
    }
    return package, lock


def record(name, version, days=14):
    return {
        "name": name,
        "version": version,
        "created_at": (NOW - timedelta(days=days)).isoformat(),
        "integrity": INTEGRITY,
        "tarball": f"https://registry.npmjs.org/{name}/-/{name.rsplit('/', 1)[-1]}-{version}.tgz",
        "fetched": NOW.isoformat(),
    }


class DocsDependencyPolicyTests(unittest.TestCase):
    def test_checks_transitives_existing_entries_and_pinned_npm(self):
        package, lock = fixture()
        seen = []

        def lookup(name, version):
            seen.append((name, version))
            return record(name, version, days=13 if name == "transitive" else 14)

        errors = policy.check(package, lock, lookup, NOW)
        self.assertEqual(len(errors), 1)
        self.assertIn("transitive@2.3.4", errors[0])
        self.assertEqual(
            set(seen), {("owned", "1.2.3"), ("transitive", "2.3.4"), ("npm", "11.6.2")}
        )

    def test_integrity_and_registry_identity_cannot_be_substituted(self):
        package, lock = fixture()
        for field, value in [
            ("integrity", "sha512-" + base64.b64encode(bytes([1]) * 64).decode()),
            ("tarball", "https://registry.npmjs.org/other/-/other-1.2.3.tgz"),
            ("name", "other"),
            ("version", "9.9.9"),
        ]:

            def lookup(name, version):
                result = record(name, version)
                if name == "owned":
                    result[field] = value
                return result

            self.assertTrue(policy.check(package, lock, lookup, NOW), field)
        for entry in [
            dict(
                lock["packages"]["node_modules/owned"],
                resolved="https://example.com/package.tgz",
            ),
            dict(
                lock["packages"]["node_modules/owned"], integrity="sha1-not-the-policy"
            ),
            dict(lock["packages"]["node_modules/owned"], link=True),
        ]:
            broken = deepcopy(lock)
            broken["packages"]["node_modules/owned"] = entry
            self.assertTrue(policy.check(package, broken, record, NOW))

    def test_unavailable_records_and_wrong_contracts_fail_closed(self):
        package, lock = fixture()

        def unavailable(*_):
            raise ValueError("unpublished version")

        self.assertTrue(policy.check(package, lock, unavailable, NOW))
        for key, value in [("lockfileVersion", 2), ("packages", None)]:
            broken = deepcopy(lock)
            broken[key] = value
            self.assertTrue(policy.check(package, broken, record, NOW))
        for name in ["npm@latest", "pnpm@11.6.2", "npm@11.6.2+unverified"]:
            broken = dict(package, packageManager=name)
            self.assertTrue(policy.check(broken, lock, record, NOW))
        broken = deepcopy(package)
        broken["dependencies"]["owned"] = "^1.2.3"
        self.assertTrue(policy.check(broken, lock, record, NOW))

    def test_registry_response_is_exact_and_missing_releases_fail(self):
        name, version = "@owned/package", "1.2.3"
        metadata = record(name, version)
        payload = {
            "name": name,
            "time": {version: metadata["created_at"]},
            "versions": {
                version: {
                    "name": name,
                    "version": version,
                    "dist": {"integrity": INTEGRITY, "tarball": metadata["tarball"]},
                }
            },
        }
        with patch.object(
            policy, "fetch", return_value=json.dumps(payload).encode()
        ) as fetch:
            self.assertEqual(
                policy.refresh(name, {version}, NOW), {f"{name}@{version}": metadata}
            )
            self.assertEqual(
                fetch.call_args.args[0], "https://registry.npmjs.org/@owned%2Fpackage"
            )
            with self.assertRaisesRegex(ValueError, "absent"):
                policy.refresh(name, {"9.9.9"}, NOW)
        payload["name"] = "other"
        with patch.object(policy, "fetch", return_value=json.dumps(payload).encode()):
            with self.assertRaisesRegex(ValueError, "different"):
                policy.refresh(name, {version}, NOW)

    def test_cached_metadata_expires_and_cannot_come_from_the_future(self):
        cached = record("owned", "1.2.3")
        self.assertTrue(policy.fresh(cached, NOW))
        for fetched in [NOW - timedelta(days=3), NOW + timedelta(seconds=1)]:
            self.assertFalse(
                policy.fresh(dict(cached, fetched=fetched.isoformat()), NOW)
            )
        self.assertFalse(policy.fresh(dict(cached, integrity="unknown"), NOW))


if __name__ == "__main__":
    unittest.main()
