import hashlib
import base58


def code_hash(code_bytes):
    return base58.b58encode(hashlib.sha256(code_bytes).digest())
