#!/usr/bin/env python3
"""Run the Android JKS generation tests against a real JDK's `keytool`, with OpenSSL as the signature verifier.

`stdlib/Android/tests/driven/jks-generation/roundtrip.jai` makes a keystore with `keytool`, reads the key back
with the Jai code and signs a message with it. It then hands the certificate's public key and the signature to
`integration_verify_signature` in a library named `sdk-verify`. This check builds that library from OpenSSL's
libcrypto (`portable-crypto-integration/` under the output directory, with `libcrypto` beside it for the toolchain's own
calls), so the signature is verified by code the toolchain does not contain.

Needs a JDK (its `keytool`) and OpenSSL with headers; it stops with an install hint when either is missing.
"""
import os
import platform
import shutil
import subprocess
import tempfile
from pathlib import Path

from developer_checks.dev_common import ROOT, cargo_binary, labeled_runner, output_dir

run = labeled_runner("Android JKS generation")

TESTS = ROOT / "stdlib/Android/tests/driven/jks-generation"

VERIFIER = r"""
#include <openssl/evp.h>
#include <openssl/x509.h>

// 1 when `signature` is a SHA-256 signature of `message` by the key in the DER SubjectPublicKeyInfo.
int integration_verify_signature(const unsigned char *public_key, unsigned long public_count,
                                 const unsigned char *message, unsigned long message_count,
                                 const unsigned char *signature, unsigned long signature_count) {
    const unsigned char *cursor = public_key;
    EVP_PKEY *key = d2i_PUBKEY(NULL, &cursor, (long) public_count);
    if (!key) return -1;
    EVP_MD_CTX *context = EVP_MD_CTX_new();
    int result = -1;
    if (context && EVP_DigestVerifyInit(context, NULL, EVP_sha256(), NULL, key) == 1 &&
        EVP_DigestVerifyUpdate(context, message, message_count) == 1) {
        result = EVP_DigestVerifyFinal(context, signature, signature_count);
    }
    EVP_MD_CTX_free(context);
    EVP_PKEY_free(key);
    return result;
}
"""


def java_bin():
    """The directory of a real JDK's `keytool`."""
    home = os.environ.get("JAVA_HOME")
    if not home and platform.system() == "Darwin":
        # /usr/bin/keytool is a stub that fails without a JDK; java_home names the installed one.
        found = subprocess.run(["/usr/libexec/java_home"], capture_output=True, text=True)
        home = found.stdout.strip() if found.returncode == 0 else None
    if home:
        directory = Path(home) / "bin"
    elif platform.system() == "Darwin":
        directory = None
    else:
        keytool = shutil.which("keytool")
        directory = Path(keytool).resolve().parent if keytool else None
    if not directory or not (directory / "keytool").exists():
        raise RuntimeError("Android JKS generation needs a JDK's keytool: install a JDK or set JAVA_HOME")
    return directory


def openssl():
    """OpenSSL's libcrypto: the compiler flags and the library directory, from pkg-config or Homebrew's keg."""
    if shutil.which("pkg-config"):
        found = subprocess.run(["pkg-config", "--cflags", "--libs", "libcrypto"], capture_output=True, text=True)
        libdir = subprocess.run(["pkg-config", "--variable=libdir", "libcrypto"], capture_output=True, text=True)
        if found.returncode == 0 and libdir.returncode == 0:
            return found.stdout.split(), Path(libdir.stdout.strip())
    if shutil.which("brew"):
        prefix = subprocess.run(["brew", "--prefix", "openssl@3"], capture_output=True, text=True).stdout.strip()
        if prefix and (Path(prefix) / "include/openssl/evp.h").exists():
            return [f"-I{prefix}/include", f"-L{prefix}/lib", "-lcrypto"], Path(prefix) / "lib"
    raise RuntimeError("Android JKS generation needs OpenSSL's libcrypto with headers: install OpenSSL "
                       "(for example `brew install openssl@3` or `apt install libssl-dev`) and pkg-config")


def build_verifier(flags, libdir):
    """The verifier library, and `libcrypto` beside it: the Android toolchain's `android-crypto/crypto`, which the SDK
    provides in a real build, is OpenSSL's here (it is the same EVP interface)."""
    directory = output_dir("portable-crypto-integration")
    directory.mkdir(parents=True, exist_ok=True)
    extension = "dylib" if platform.system() == "Darwin" else "so"
    source = directory / "sdk-verify.c"
    source.write_text(VERIFIER)
    link = directory / f"libcrypto.{extension}"
    link.unlink(missing_ok=True)
    link.symlink_to((libdir / f"libcrypto.{extension}").resolve())
    library = directory / f"libsdk-verify.{extension}"
    command = ["cc", "-O1", "-fPIC", "-shared", str(source), "-o", str(library), *flags]
    if platform.system() == "Darwin":
        command += ["-Wl,-install_name,@rpath/libsdk-verify.dylib"]
    subprocess.run(command, check=True)


def main():
    jaic = cargo_binary("jaic")
    java = str(java_bin())
    build_verifier(*openssl())
    with tempfile.TemporaryDirectory() as scratch:
        scratch = Path(scratch)
        for name, arguments in [
            ("main", lambda: [str(scratch / "absent.jks")]),  # logs each refusal on stderr
            ("positive", lambda: [java, str(scratch / "positive.jks")]),
            ("roundtrip", None),
        ]:
            source = TESTS / f"{name}.jai"
            executable = output_dir(f"android-jks-{name}")
            run([jaic, source, "-exe", executable])
            if arguments:
                run([executable, *arguments()], "ok\n", stderr=None if name == "main" else "")
                continue
            for algorithm in ("EC", "RSA", "DSA"):
                run([executable, java, str(scratch / f"roundtrip-{algorithm}.jks"), algorithm], "ok\n")
    print("Android JKS generation matched keytool and OpenSSL for EC, RSA and DSA keys")


if __name__ == "__main__":
    main()
