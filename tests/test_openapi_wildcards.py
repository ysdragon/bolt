import pytest
from conftest import unwrap


def test_openapi_named_wildcard_path_param(client):
    r = client.get("/openapi.json")
    assert r.status_code == 200
    spec = r.json()
    assert "/files/{path:.*}" in spec["paths"]
    params = spec["paths"]["/files/{path:.*}"]["get"]["parameters"]
    assert any(p["name"] == "path" and p["in"] == "path" and p.get("required")
               for p in params)


def test_openapi_mixed_wildcard_path_params(client):
    r = client.get("/openapi.json")
    assert r.status_code == 200
    spec = r.json()
    path_key = "/users/{id}/settings/{section:.*}"
    assert path_key in spec["paths"]
    params = spec["paths"][path_key]["get"]["parameters"]
    names = {p["name"] for p in params if p["in"] == "path"}
    assert names == {"id", "section"}
