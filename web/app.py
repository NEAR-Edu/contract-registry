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


def get_job_artifacts(job_number):
    return req.get(
        f"https://circleci.com/api/v2/project/{circleci_project_slug}/{job_number}/artifacts",
        headers={"Circle-Token": circleci_api_key},
    ).json()


@app.get("/test/<job_number>")
def test(job_number):
    return get_job_artifacts(job_number), 200


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
