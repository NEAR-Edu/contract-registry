from dataclasses import dataclass
from typing import Dict, List
from dataclasses_json import dataclass_json
from os import environ
from dotenv import load_dotenv

import aiohttp
from aiohttp import web
import asyncio


import circleci_verify

load_dotenv()

routes = web.RouteTableDef()

registry_cache_dir = environ["REGISTRY_CACHE_DIR"]
circleci_webhook_secret = environ["CIRCLECI_WEBHOOK_SECRET"]
circleci_api_key = environ["CIRCLECI_API_KEY"]
circleci_job_name = environ["CIRCLECI_JOB_NAME"]
circleci_project_slug = environ["CIRCLECI_PROJECT_SLUG"]

auth_headers = {"Circle-Token": circleci_api_key}


async def auth_get_text(session: aiohttp.ClientSession, url):
    async with session.get(url, headers=auth_headers) as request:
        return await request.text()


async def auth_get_raw(session: aiohttp.ClientSession, url):
    async with session.get(url, headers=auth_headers) as request:
        return await request.read()


async def auth_get_json(session: aiohttp.ClientSession, url):
    async with session.get(url, headers=auth_headers) as request:
        return await request.json(content_type=None)


async def get_job_artifacts(session: aiohttp.ClientSession, job_number):
    res = await auth_get_json(
        session,
        f"https://circleci.com/api/v2/project/{circleci_project_slug}/{job_number}/artifacts",
    )

    d = {}

    for item in res["items"]:
        d[item["path"]] = item["url"]

    return d


async def parallel_artifacts(
    session: aiohttp.ClientSession, artifacts: Dict[str, str], artifact_paths: List[str]
) -> List[str]:
    urls = [artifacts[path] for path in artifact_paths]
    requests = [auth_get_text(session, url) for url in urls]
    return await asyncio.gather(*requests)


@dataclass_json
@dataclass
class VerificationMetadata:
    repo: str
    remote: str
    branch: str
    commit_hash: str

    @staticmethod
    async def assemble(session, artifacts):
        artifact_contents = await parallel_artifacts(
            session,
            artifacts,
            [
                "git/repo.txt",
                "git/remote.txt",
                "git/branch.txt",
                "git/commit.txt",
            ],
        )
        return VerificationMetadata(*[text.strip() for text in artifact_contents])


@routes.get("/test/{job_number}")
async def test(request: web.Request):
    job_number = request.match_info["job_number"]

    async with aiohttp.ClientSession() as session:
        artifacts = await get_job_artifacts(session, job_number)
        metadata, code_bytes = await asyncio.gather(
            VerificationMetadata.assemble(session, artifacts),
            auth_get_raw(session, artifacts["out/out.wasm"]),
        )
        print(metadata.to_dict())
        from code_hash import code_hash

        print(code_hash(code_bytes))


@routes.post("/webhook")
async def webhook(request: web.Request):
    if not circleci_verify.verify_signature(
        circleci_webhook_secret,
        request.headers,
        request.data,
    ):
        return {"error": "Unauthorized"}, 401

    body = request.json()

    print(body)

    if not (
        body["type"] == "job-completed"
        and body["job"]["name"] == circleci_job_name
        and body["workflow"]["status"] == "success"
    ):
        return {"error": "Malformed request"}, 400

    job_number = body["job"]["number"]

    async with aiohttp.ClientSession() as session:
        artifacts = await get_job_artifacts(session, job_number)
        [metadata, code_bytes] = await asyncio.gather(
            VerificationMetadata.assemble(session, artifacts),
            auth_get(session, artifacts["out/out.wasm"]),
        )
        print(metadata.to_dict())
        print(code_bytes)


if __name__ == "__main__":
    app = web.Application()
    app.add_routes(routes)
    web.run_app(app)
