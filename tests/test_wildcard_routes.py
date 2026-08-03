import pytest
from conftest import unwrap


def test_named_wildcard_matches_slashes(client):
    r = client.get("/files/docs/guides/intro.md")
    assert r.status_code == 200
    assert unwrap(r.json())["path"] == "docs/guides/intro.md"


def test_named_wildcard_single_segment(client):
    r = client.get("/files/notes.txt")
    assert r.status_code == 200
    assert unwrap(r.json())["path"] == "notes.txt"


def test_wildcard_with_regular_param(client):
    r = client.get("/users/123/settings/notifications/email")
    assert r.status_code == 200
    data = unwrap(r.json())
    assert data["id"] == "123"
    assert data["section"] == "notifications/email"


def test_anonymous_catch_all(client):
    r = client.get("/anything/here/at/all")
    assert r.status_code == 200
    assert unwrap(r.json())["matched"] == "catch-all"


def test_anonymous_catch_all_single_segment(client):
    r = client.get("/unknown")
    assert r.status_code == 200
    assert unwrap(r.json())["matched"] == "catch-all"


def test_catch_all_does_not_shadow_docs_endpoints(client):
    r = client.get("/openapi.json")
    assert r.status_code == 200
    assert "openapi" in r.json()
    r = client.get("/docs/")
    assert r.status_code == 200
    assert "swagger" in r.text.lower() or "openapi" in r.text.lower()


def test_catch_all_still_matches_other_paths(client):
    r = client.get("/openapi.json")
    assert r.status_code == 200
    assert "openapi" in r.json()
    r = client.get("/anything/else")
    assert r.status_code == 200
    assert unwrap(r.json())["matched"] == "catch-all"
