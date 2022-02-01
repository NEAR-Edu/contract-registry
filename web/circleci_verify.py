import hmac


def verify_signature(secret, headers, body):
    # get the v1 signature from the `circleci-signature` header
    signature_from_header = {
        k: v
        for k, v in [
            pair.split("=") for pair in headers["circleci-signature"].split(",")
        ]
    }["v1"]

    # Run HMAC-SHA256 on the request body using the configured signing secret
    valid_signature = hmac.new(
        bytes(secret, "utf-8"), bytes(body, "utf-8"), "sha256"
    ).hexdigest()

    # use constant time string comparison to prevent timing attacks
    return hmac.compare_digest(valid_signature, signature_from_header)
