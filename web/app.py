from dataclasses import dataclass
from dataclasses_json import dataclass_json
from os import environ

import requests as req
from flask import Flask, request

import circleci_verify

app = Flask(__name__)

registry_cache_dir = environ["REGISTRY_CACHE_DIR"]
circleci_webhook_secret = environ["CIRCLECI_WEBHOOK_SECRET"]
circleci_api_key = environ["CIRCLECI_API_KEY"]
circleci_job_name = environ["CIRCLECI_JOB_NAME"]
circleci_project_slug = environ["CIRCLECI_PROJECT_SLUG"]

auth_headers = {"Circle-Token": circleci_api_key}


def auth_get(url):
    return req.get(url, headers=auth_headers)


def get_job_artifacts(job_number):
    res = auth_get(
        f"https://circleci.com/api/v2/project/{circleci_project_slug}/{job_number}/artifacts",
    ).json()

    d = {}

    for item in res["items"]:
        d[item["path"]] = item["url"]

    return d


@dataclass_json
@dataclass
class VerificationMetadata:
    repo: str
    remote: str
    branch: str
    commit_hash: str

    @staticmethod
    def assemble(artifacts):
        return VerificationMetadata(
            auth_get(artifacts["git/repo.txt"]).text.strip(),
            auth_get(artifacts["git/remote.txt"]).text.strip(),
            auth_get(artifacts["git/branch.txt"]).text.strip(),
            auth_get(artifacts["git/commit.txt"]).text.strip(),
        )


@app.get("/test/<job_number>")
def test(job_number):
    artifacts = get_job_artifacts(job_number)
    m = VerificationMetadata.assemble(artifacts)
    return m.to_dict()


@app.post("/webhook")
def webhook():
    if not circleci_verify.verify_signature(
        circleci_webhook_secret,
        request.headers,
        request.data,
    ):
        return {"error": "Unauthorized"}, 401

    body = request.get_json()

    print(body)

    if not (
        body["type"] == "job-completed"
        and body["job"]["name"] == circleci_job_name
        and body["workflow"]["status"] == "success"
    ):
        return {"error": "Malformed request"}, 400


if __name__ == "__main__":
    app.run()
